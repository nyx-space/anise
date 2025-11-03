import os
import subprocess

# Add the site-packages directory to the PYTHONPATH
site_packages_path = "/home/jules/.pyenv/versions/3.12.12/lib/python3.12/site-packages"
os.environ["PYTHONPATH"] = f"{os.environ.get('PYTHONPATH', '')}:{site_packages_path}"

# Run the stub generation script
subprocess.run(["python3", "anise-py/generate_stubs.py", "anise", "anise-py/anise.pyi", "--ruff"])
