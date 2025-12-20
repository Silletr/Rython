import os
import sys
import subprocess
import shutil

def run_cmd(cmd, description):
    print(f"[*] {description}...")
    try:
        subprocess.run(cmd, check=True, shell=isinstance(cmd, str))
    except subprocess.CalledProcessError as e:
        print(f"[!] Error: {description} failed.")
        sys.exit(1)

def main():
    # 1. Setup paths
    base_dir = os.path.dirname(os.path.abspath(__file__))
    dist_dir = os.path.join(base_dir, "dist")
    
    if os.path.exists(dist_dir):
        shutil.rmtree(dist_dir)
    os.makedirs(dist_dir)

    # 2. Build Rust Compiler and Library
    print("--- Building Rython Core (Rust) ---")
    run_cmd(["cargo", "build", "--release"], "Compiling Rust binary and library")

    # 3. Copy Rust artifacts to dist/
    target_dir = os.path.join(base_dir, "target", "release")
    
    # Executable
    names = ["rythonc", "rythonc.exe"]
    for name in names:
        src = os.path.join(target_dir, name)
        if os.path.exists(src):
            shutil.copy2(src, dist_dir)
            print(f"[+] Compiled: {os.path.join('dist', name)}")
    
    # Shared Library (for runtime linking)
    # Handle different OS extensions
    extensions = [".so", ".dll", ".dylib"]
    for ext in extensions:
        lib_file = f"librython_jit{ext}"
        src = os.path.join(target_dir, lib_file)
        if os.path.exists(src):
            shutil.copy2(src, dist_dir)
            print(f"[+] Copied: {os.path.join('dist', lib_file)}")
        
        # Also check without 'lib' prefix (Windows)
        lib_file_win = f"rython_jit{ext}"
        src_win = os.path.join(target_dir, lib_file_win)
        if os.path.exists(src_win):
            shutil.copy2(src_win, dist_dir)
            print(f"[+] Copied: {os.path.join('dist', lib_file_win)}")

    # 4. Build Python CLI (ryc)
    print("\n--- Building RYC CLI (Python) ---")
    
    # Check if pyinstaller is available
    pyinstaller_exists = subprocess.call(["which", "pyinstaller"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0
    if not pyinstaller_exists:
        print("[!] PyInstaller not found. Attempting to install...")
        run_cmd([sys.executable, "-m", "pip", "install", "pyinstaller", "rich", "--break-system-packages"], "Installing PyInstaller and dependencies")

    # Run PyInstaller
    # We use --onefile and --name ryc
    pyinst_cmd = [
        sys.executable, "-m", "PyInstaller",
        "--onefile",
        "--name", "charge",
        "--distpath", dist_dir,
        "--workpath", os.path.join(base_dir, "build_py"),
        "--specpath", base_dir,
        os.path.join(base_dir, "ryc.py")
    ]
    run_cmd(pyinst_cmd, "Bundling ryc.py into a standalone binary")

    print("\n" + "="*40)
    print("RYTHON BUILD COMPLETE!")
    print(f"Binaries available in: {dist_dir}")
    print("="*40)

if __name__ == "__main__":
    main()
