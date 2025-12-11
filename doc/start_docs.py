#!/usr/bin/env python3
"""
Script to start the MkDocs documentation server.
Automatically sets up a virtual environment and installs dependencies if needed.
"""

import os
import subprocess
import sys
import webbrowser
from pathlib import Path
import time
import platform

def main():
    # Get the directory containing this script
    script_dir = Path(__file__).parent.absolute()
    venv_dir = script_dir / "venv"
    requirements_file = script_dir / "docs" / "requirements.txt"
    
    # Determine the correct Python executable path
    if platform.system() == "Windows":
        python_exe = venv_dir / "Scripts" / "python.exe"
        pip_exe = venv_dir / "Scripts" / "pip.exe"
    else:
        python_exe = venv_dir / "bin" / "python"
        pip_exe = venv_dir / "bin" / "pip"
    
    # Create virtual environment if it doesn't exist
    if not venv_dir.exists():
        print("Creating virtual environment...")
        subprocess.run([sys.executable, "-m", "venv", str(venv_dir)], check=True)
        print("Virtual environment created.")
    
    # Install/upgrade dependencies
    print("Installing dependencies...")
    subprocess.run([str(pip_exe), "install", "-q", "--upgrade", "pip"], check=True)
    subprocess.run([str(pip_exe), "install", "-q", "-r", str(requirements_file)], check=True)
    print("Dependencies installed.")
    
    # Start MkDocs server
    print("\nStarting MkDocs server...")
    print("Documentation will be available at: http://localhost:8000")
    print("Press Ctrl+C to stop the server.\n")
    
    # Open browser after a short delay
    def open_browser():
        time.sleep(2)
        webbrowser.open("http://localhost:8000")
    
    import threading
    threading.Thread(target=open_browser, daemon=True).start()
    
    # Run mkdocs serve from the script directory
    try:
        subprocess.run(
            [str(python_exe), "-m", "mkdocs", "serve"],
            cwd=str(script_dir),
            check=True
        )
    except KeyboardInterrupt:
        print("\nShutting down MkDocs server...")
        sys.exit(0)

if __name__ == "__main__":
    main()
