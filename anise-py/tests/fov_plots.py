import numpy as np
import plotly.graph_objects as go
from plotly.subplots import make_subplots

from anise.rotation import Quaternion

# --- CONSTANTS ---
R_MOON = 1737.4  # km
H_SC = 100.0  # Altitude for visualization (km)
SC_POS = np.array([R_MOON + H_SC, 0, 0])  # Spacecraft Position (Body +X)
TARGET_POS = np.array([R_MOON, 0, 0])  # Target (Nadir)

# FOV DEFINITION
X_HALF_DEG = 15.0
Y_HALF_DEG = 10.0
FOV_DIST = 150.0  # Length of pyramid edges for visualization

# --- HELPER FUNCTIONS ---


def get_rotation_matrix(angle_deg, axis):
    """Returns 3x3 rotation matrix (Passive/Coordinate transform)."""
    rad = np.deg2rad(angle_deg)
    if axis == "x":
        return Quaternion.about_x(rad, 0, 1).to_dcm().rot_mat
    elif axis == "y":
        return Quaternion.about_y(rad, 0, 1).to_dcm().rot_mat
    else:
        return Quaternion.about_z(rad, 0, 1).to_dcm().rot_mat


def intersect_sphere(origin, direction, radius):
    """Ray-Sphere Intersection."""
    direction = direction / np.linalg.norm(direction)
    a = 1.0
    b = 2 * np.dot(origin, direction)
    c = np.dot(origin, origin) - radius**2
    delta = b**2 - 4 * a * c
    if delta < 0:
        return None
    sol1 = (-b - np.sqrt(delta)) / 2
    sol2 = (-b + np.sqrt(delta)) / 2
    if sol1 <= 0 and sol2 <= 0:
        return None
    return origin + min(sol1, sol2) * direction


def build_fov_pyramid(R_body_to_sensor):
    """
    Returns vertices for the FOV pyramid in the Body Frame.
    R_body_to_sensor: Rotation from Body to Sensor (Passive).
    """
    # 1. Define Rays in Sensor Frame (Z = Boresight)
    # Tan(theta) gives x/z. Let z=1.
    tx = np.tan(np.deg2rad(X_HALF_DEG))
    ty = np.tan(np.deg2rad(Y_HALF_DEG))

    # Sensor Frame Vectors (Unit length)
    vectors_s = [
        np.array([0, 0, 1]),  # Boresight
        np.array([-tx, ty, 1]),  # TL
        np.array([tx, ty, 1]),  # TR
        np.array([tx, -ty, 1]),  # BR
        np.array([-tx, -ty, 1]),  # BL
    ]
    vectors_s = [v / np.linalg.norm(v) for v in vectors_s]

    # 2. Transform to Body Frame
    # v_body = R.T @ v_sensor (since R is Body->Sensor)
    R_sens_to_body = R_body_to_sensor.T
    vectors_b = [R_sens_to_body @ v for v in vectors_s]

    # 3. Project to Surface or Fixed Distance
    points_b = []
    for v in vectors_b:
        p = intersect_sphere(SC_POS, v, R_MOON)
        if p is None:
            # If missing (looking away), draw fixed length
            p = SC_POS + v * FOV_DIST * 3
        points_b.append(p)

    return points_b  # [Boresight, TL, TR, BR, BL]


# --- SETUP TEST CASES ---
# Nominal: -90 deg Y rotation. Inst Z aligns with Body -X.
R_nom = get_rotation_matrix(-90, "y")

# Case 1: Edge. -105 deg Y.
R_edge = get_rotation_matrix(-105, "y")

# Case 2: Outside. Nominal + 12 deg Pitch (Inst X).
# Rot = Pitch_Inst * Base_Body = R_x(12) * R_nom
R_outside = get_rotation_matrix(12, "x") @ R_nom

# Case 3: Zenith. +90 deg Y.
R_zenith = get_rotation_matrix(90, "y")

cases = [
    ("Nominal (Nadir)", R_nom, "green"),
    ("Edge Case (-15 Pitch)", R_edge, "orange"),
    ("Outside Height (12 Roll)", R_outside, "red"),
    ("Zenith (Away)", R_zenith, "black"),
]

# --- PLOTLY FIGURE ---
fig = make_subplots(
    rows=2,
    cols=2,
    specs=[
        [{"type": "scene"}, {"type": "scene"}],
        [{"type": "scene"}, {"type": "scene"}],
    ],
    subplot_titles=[c[0] for c in cases],
)

# Moon Sphere Wireframe
u = np.linspace(0, 2 * np.pi, 30)
v = np.linspace(0, np.pi, 15)
x_sph = R_MOON * np.outer(np.cos(u), np.sin(v))
y_sph = R_MOON * np.outer(np.sin(u), np.sin(v))
z_sph = R_MOON * np.outer(np.ones(np.size(u)), np.cos(v))

for i, (title, R, color) in enumerate(cases):
    row = (i // 2) + 1
    col = (i % 2) + 1

    # 1. Moon Surface
    fig.add_trace(
        go.Mesh3d(
            x=x_sph.flatten(),
            y=y_sph.flatten(),
            z=z_sph.flatten(),
            color="gray",
            opacity=0.1,
            alphahull=0,
            showscale=False,
        ),
        row=row,
        col=col,
    )

    # 2. Spacecraft
    fig.add_trace(
        go.Scatter3d(
            x=[SC_POS[0]],
            y=[SC_POS[1]],
            z=[SC_POS[2]],
            mode="markers",
            marker=dict(size=5, color="blue"),
            name="LRO",
        ),
        row=row,
        col=col,
    )

    # 3. Target (Nadir)
    fig.add_trace(
        go.Scatter3d(
            x=[TARGET_POS[0]],
            y=[TARGET_POS[1]],
            z=[TARGET_POS[2]],
            mode="markers",
            marker=dict(size=5, color="magenta", symbol="x"),
            name="Target",
        ),
        row=row,
        col=col,
    )

    # 4. FOV Pyramid
    pts = build_fov_pyramid(R)
    # Vertices: SC_POS, pts[1]...pts[4]
    # Faces for Mesh3d
    # We define vertices [SC, TL, TR, BR, BL]
    bx, by, bz = SC_POS
    vx = [bx] + [p[0] for p in pts[1:]]
    vy = [by] + [p[1] for p in pts[1:]]
    vz = [bz] + [p[2] for p in pts[1:]]

    # Indices for triangles: (0,1,2), (0,2,3), (0,3,4), (0,4,1)
    # Plus the cap (1,2,3,4) - simplifed as (1,2,3) and (1,3,4)
    i_ind = [0, 0, 0, 0, 1, 1]
    j_ind = [1, 2, 3, 4, 2, 3]
    k_ind = [2, 3, 4, 1, 3, 4]

    fig.add_trace(
        go.Mesh3d(
            x=vx,
            y=vy,
            z=vz,
            i=i_ind,
            j=j_ind,
            k=k_ind,
            color=color,
            opacity=0.3,
            showscale=False,
            name="FOV",
        ),
        row=row,
        col=col,
    )

    # 5. Boresight Line
    p_bore = pts[0]
    fig.add_trace(
        go.Scatter3d(
            x=[SC_POS[0], p_bore[0]],
            y=[SC_POS[1], p_bore[1]],
            z=[SC_POS[2], p_bore[2]],
            mode="lines",
            line=dict(color="black", width=4, dash="dash"),
            name="Boresight",
        ),
        row=row,
        col=col,
    )

    # Camera Settings
    camera = dict(
        up=dict(x=0, y=0, z=1),
        center=dict(x=0, y=0, z=0),
        eye=dict(x=0.5, y=-1.5, z=0.5),  # Look from side/back
    )
    fig.update_scenes(camera=camera, row=row, col=col)

    # Axis Limits (Zoom in on relevant patch)
    if "Zenith" in title:
        limit = 3000
        center = 0
    else:
        limit = 300
        center = R_MOON

    fig.update_scenes(
        xaxis=dict(range=[center - limit, center + limit], title="Radial"),
        yaxis=dict(range=[-limit, limit], title="In-Track"),
        zaxis=dict(range=[-limit, limit], title="Cross-Track"),
        aspectmode="cube",
        row=row,
        col=col,
    )

fig.update_layout(
    height=1024, width=1280, title_text="LRO Camera FOV Verification", showlegend=False
)

fig.show()
