import typing

def convert_fk(fk_file_path: str, anise_output_path: str, show_comments: bool=None, overwrite: bool=None) -> None:
    """Converts a KPL/FK file, that defines frame constants like fixed rotations, and frame name to ID mappings into the EulerParameterDataSet equivalent ANISE file.
KPL/FK files must be converted into "PCA" (Planetary Constant ANISE) files before being loaded into ANISE."""

def convert_tpc(pck_file_path: str, gm_file_path: str, anise_output_path: str, overwrite: bool=None) -> None:
    """Converts two KPL/TPC files, one defining the planetary constants as text, and the other defining the gravity parameters, into the PlanetaryDataSet equivalent ANISE file.
KPL/TPC files must be converted into "PCA" (Planetary Constant ANISE) files before being loaded into ANISE."""