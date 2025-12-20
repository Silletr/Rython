import os
import sys
import subprocess
import argparse
import bridge
from bridge import rython_jit
from rich.console import Console
from rich.panel import Panel
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.text import Text
from rich import print as rprint

# Try to import tomllib (Python 3.11+) or tomli
try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib
    except ImportError:
        tomllib = None

console = Console()

BANNER = r"""
[bold cyan]
  _____         _   _                      
 |  __ \       | | | |                     
 | |__) | _   _| |_| |__    ___   _ __     
 |  _  / | | | | __| '_ \  / _ \ | '_ \    
 | | \ \ | |_| | |_| | | || (_) || | | |   
 |_|  \_\ \__, |\__|_| |_| \___/ |_| |_|   
           __/ |                           
          |___/                            
[/bold cyan]
[dim italic]High-Performance LLVM-Based Compiler[/dim italic]
"""

CHARGE_TEMPLATE = """[package]
name = "{name}"
version = "0.1.0"
entry = "main.ry"

[build]
linker = "clang"
"""

def get_config():
    if not os.path.exists("Charge.toml"):
        return None
    
    if tomllib is None:
        console.print("[bold yellow]Warning:[/bold yellow] 'Charge.toml' found but 'tomllib/tomli' not installed.", style="yellow")
        console.print("Please install with: [bold cyan]sudo apt install python3-tomli[/bold cyan] (on Ubuntu)")
        return None

    try:
        with open("Charge.toml", "rb") as f:
            return tomllib.load(f)
    except Exception as e:
        console.print(f"[bold red]Error reading Charge.toml:[/bold red] {e}", style="red")
        return None

def print_banner():
    console.print(BANNER)
    lib_path = getattr(rython_jit, "__file__", "unknown")
    console.print(f"[dim]Runtime Loaded: {lib_path}[/dim]\n")

def compile_rython(input_file, output_executable, save_obj=False):
    if not os.path.exists(input_file):
        console.print(f"[bold red]Error:[/bold red] File [italic]{input_file}[/italic] not found", style="red")
        sys.exit(1)

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console,
        transient=True
    ) as progress:
        
        # 1. Compile to Object File
        task1 = progress.add_task("[cyan]Compiling Rython source...", total=None)
        
        # Use RAM for temporary object file if possible (/dev/shm is typical for WSL/Linux)
        if os.path.exists("/dev/shm") and not save_obj:
            import tempfile
            temp_o = tempfile.NamedTemporaryFile(dir="/dev/shm", suffix=".o", delete=False)
            obj_file = temp_o.name
            temp_o.close()
        else:
            obj_file = input_file + ".o"

        try:
            # We need to make sure 'code' is available
            with open(input_file, 'r') as f:
                code_content = f.read().strip()
            rython_jit.compile_to_object(code_content, obj_file)
        except Exception as e:
            progress.stop()
            console.print(Panel(f"[bold red]Compilation Error:[/bold red]\n{e}", title="Error", border_style="red"))
            if os.path.exists(obj_file) and obj_file.startswith("/dev/shm"):
                os.remove(obj_file)
            sys.exit(1)
        # 2. Find the runtime library
        lib_dir = os.path.dirname(rython_jit.__file__)
        lib_name = "rython_jit"

        # 3. Choose Linker
        linker = "clang"
        if subprocess.call(["which", "clang"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) != 0:
            if subprocess.call(["which", "gcc"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
                linker = "gcc"
            else:
                progress.stop()
                console.print("[bold red]Error:[/bold red] No linker found (clang or gcc).", style="red")
                sys.exit(1)

        # 4. Get Python linking flags
        try:
            py_libs = subprocess.check_output(["python3-config", "--libs", "--embed"]).decode().strip().split()
            py_ldflags = subprocess.check_output(["python3-config", "--ldflags", "--embed"]).decode().strip().split()
        except:
            py_libs = subprocess.check_output(["python3-config", "--libs"]).decode().strip().split()
            py_ldflags = subprocess.check_output(["python3-config", "--ldflags"]).decode().strip().split()

        # 5. Link
        cmd = [
            linker,
            obj_file,
            "-L" + lib_dir,
            "-l" + lib_name,
            "-lgc",
        ] + py_ldflags + py_libs + [
            "-Wl,-rpath," + lib_dir,
            "-o", output_executable
        ]
        
        try:
            subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        except subprocess.CalledProcessError as e:
            progress.stop()
            console.print(Panel(f"[bold red]Linking Failed:[/bold red]\n{e.stderr.decode()}", title="Linker Error", border_style="red"))
            if not save_obj and os.path.exists(obj_file):
                os.remove(obj_file)
            sys.exit(1)
        finally:
            if not save_obj and os.path.exists(obj_file):
                os.remove(obj_file)
            elif save_obj and obj_file.startswith("/dev/shm"):
                # Move from RAM to current dir if user wanted to save it
                dest = input_file + ".o"
                import shutil
                shutil.move(obj_file, dest)
                obj_file = dest

    msg = f"[bold green]✓[/bold green] [cyan]{input_file}[/cyan] -> [bold yellow]{output_executable}[/bold yellow]"
    if save_obj:
        msg += f" (Object: [bold blue]{obj_file}[/bold blue])"
    console.print(msg)
    return True

def cmd_init(args):
    print_banner()
    name = os.path.basename(os.getcwd())
    if os.path.exists("Charge.toml"):
        console.print("[bold red]Error:[/bold red] Charge.toml already exists in this directory.", style="red")
        return
    
    with open("Charge.toml", "w") as f:
        f.write(CHARGE_TEMPLATE.format(name=name))
    
    if not os.path.exists("main.ry"):
        with open("main.ry", "w") as f:
            f.write('function main() -> int:\n    rython_print_str("Hello from Charge!")\n    return 0\n')
    
    console.print(Panel(f"Project [bold cyan]{name}[/bold cyan] initialized successfully!", border_style="green"))

def cmd_build(args):
    print_banner()
    config = get_config()
    save_obj = getattr(args, 'save_obj', False)
    if config:
        name = config.get("package", {}).get("name", "a.out")
        entry = config.get("package", {}).get("entry", "main.ry")
        compile_rython(entry, name, save_obj=save_obj)
    elif args.file:
        compile_rython(args.file, args.output, save_obj=save_obj)
    else:
        console.print("[bold red]Error:[/bold red] No Charge.toml found and no input file provided.", style="red")
        sys.exit(1)

def cmd_run(args):
    print_banner()
    config = get_config()
    save_obj = getattr(args, 'save_obj', False)
    if config:
        name = config.get("package", {}).get("name", "a.out")
        entry = config.get("package", {}).get("entry", "main.ry")
        if compile_rython(entry, name, save_obj=save_obj):
            # Run the produced binary
            console.print(f"[bold magenta]▶ Running[/bold magenta] {name}...")
            console.print("[dim]" + "─" * 40 + "[/dim]")
            subprocess.run([f"./{name}"])
            console.print("[dim]" + "─" * 40 + "[/dim]")
    elif args.file:
        if compile_rython(args.file, args.output):
            console.print(f"[bold magenta]▶ Running[/bold magenta] {args.output}...")
            console.print("[dim]" + "─" * 40 + "[/dim]")
            subprocess.run([f"./{args.output}"])
            console.print("[dim]" + "─" * 40 + "[/dim]")
    else:
        console.print("[bold red]Error:[/bold red] No Charge.toml found and no input file provided.", style="red")
        sys.exit(1)

def print_custom_help(parser):
    from rich.table import Table
    
    print_banner()
    console.print(r"[bold white]Usage:[/bold white] [cyan]ryc[/cyan] [yellow]<command>[/yellow] [dim]\[options][/dim]\n")
    
    # Subcommands Table
    table = Table(box=None, padding=(0, 2), show_header=False)
    table.add_column("Command", style="yellow", no_wrap=True)
    table.add_column("Description", style="white")
    
    table.add_row("init", "Initialize a new Rython project in the current directory")
    table.add_row("build", "Compile the project (Charge.toml) or a specific file")
    table.add_row("run", "Compile and execute the project or a specific file")
    
    console.print(Panel(table, title="[bold cyan]Available Commands[/bold cyan]", title_align="left", border_style="cyan"))
    
    # Options
    console.print("\n[bold white]General Options:[/bold white]")
    console.print("  [yellow]-h, --help[/yellow]         Show this beautiful help message")
    console.print("  [yellow]-s, --save-obj[/yellow]     Keep the intermediate .o file")
    
    console.print("\n[bold white]Examples:[/bold white]")
    console.print("  [dim]$[/dim] [cyan]ryc[/cyan] build [yellow]-s[/yellow]")
    console.print("  [dim]$[/dim] [cyan]ryc[/cyan] run [dim]hello.ry[/dim]")
    console.print("")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Rython Compiler & Project Manager", add_help=False)
    subparsers = parser.add_subparsers(dest="command")

    # Build command
    build_parser = subparsers.add_parser("build", add_help=False)
    build_parser.add_argument("file", nargs="?", help="Optional file to compile (overrides Charge.toml)")
    build_parser.add_argument("-o", "--output", default="a.out", help="Output executable name")
    build_parser.add_argument("-s", "--save-obj", action="store_true", help="Save the intermediate object file")
    build_parser.add_argument("-h", "--help", action="store_true")

    # Run command
    run_parser = subparsers.add_parser("run", add_help=False)
    run_parser.add_argument("file", nargs="?", help="Optional file to compile and run")
    run_parser.add_argument("-o", "--output", default="a.out", help="Output executable name")
    run_parser.add_argument("-s", "--save-obj", action="store_true", help="Save the intermediate object file")
    run_parser.add_argument("-h", "--help", action="store_true")

    # Init command
    init_parser = subparsers.add_parser("init", add_help=False)
    init_parser.add_argument("-h", "--help", action="store_true")

    # Legacy support / Help
    parser.add_argument("-h", "--help", action="store_true")

    if len(sys.argv) == 1:
        print_custom_help(parser)
        sys.exit(0)
        
    args, unknown = parser.parse_known_args()

    if args.help or (args.command and hasattr(args, 'help') and args.help):
        if args.command == "build":
            console.print(r"\n[bold yellow]ryc build[/bold yellow] [dim]\[file] [-o output][/dim]")
            console.print("Compiles a project based on [bold]Charge.toml[/bold] or a single [bold].ry[/bold] file.\n")
        elif args.command == "run":
            console.print(r"\n[bold yellow]ryc run[/bold yellow] [dim]\[file] [-o output][/dim]")
            console.print("Builds and executes the project or a single [bold].ry[/bold] file.\n")
        elif args.command == "init":
            console.print("\n[bold yellow]ryc init[/bold yellow]")
            console.print("Generates a [bold]Charge.toml[/bold] and a template [bold]main.ry[/bold].\n")
        else:
            print_custom_help(parser)
        sys.exit(0)

    if args.command == "init":
        cmd_init(args)
    elif args.command == "build":
        cmd_build(args)
    elif args.command == "run":
        cmd_run(args)
    elif not args.command and len(sys.argv) > 1:
        # Fallback for legacy "ryc.py file.ry" usage
        compile_rython(sys.argv[1], "a.out")
    else:
        print_custom_help(parser)



