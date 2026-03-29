import random
import sys
import time

import typer
from rich import print as rprint
from rich.console import Console
from rich.panel import Panel
from rich.progress import (
    BarColumn,
    DownloadColumn,
    Progress,
    SpinnerColumn,
    TextColumn,
    TimeRemainingColumn,
    TransferSpeedColumn,
)
from rich.table import Table

# 1. 判断是否是真实的终端 (TTY)
# is_tty = True  -> 你在 VSCode/CMD 里直接跑
# is_tty = False -> 你在 Pueue/Docker/管道 里跑
is_tty = sys.stdout.isatty()

# 2. 如果检测到不是终端（比如在 Pueue 里），且没有被显式禁用缓冲
#    才强制修改缓冲策略。这样绝对不会影响你本地运行的效果。
if not is_tty:
    # 强制让 Python 只要有换行就输出，解决 Pueue "看不到日志" 的问题
    sys.stdout.reconfigure(line_buffering=True)

# 3. 初始化 Console
# force_terminal=not is_tty:
#    - 如果是真终端(Local)，让 Rich 自己检测（它会处理得很好），不要瞎指挥。
#    - 如果是Pueue(Background)，强制开启终端模式，保留颜色。
# width=...:
#    - 后台运行时给个固定宽度，防止进度条乱换行。
console = Console(
    force_terminal=True if not is_tty else None,
    width=100 if not is_tty else None,
)

ESC = "\x1b"
OSC = "\x1b]"
ST = "\x1b\\"

app = typer.Typer(
    help=(
        "一个用于观察终端输出如何被后台系统记录的样本 CLI。"
        "不同子命令分别覆盖 plain text、CR 重绘、ANSI、cursor motion、OSC、alt-screen 和 Rich progress。"
    )
)


def scaled_sleep(base_seconds: float, scale: float) -> None:
    if scale <= 0:
        return
    time.sleep(base_seconds * scale)


def emit(text: str) -> None:
    sys.stdout.write(text)
    sys.stdout.flush()


@app.command()
def install(
    package_name: str = typer.Argument(..., help="模拟要安装的包名"),
    delay_scale: float = typer.Option(1.0, min=0.0, help="把所有 sleep 按比例缩放，便于加速测试。"),
    seed: int = typer.Option(7, help="控制进度推进随机性的种子，保证复现实验。"),
):
    """
    模拟一个现代化的包安装过程，展示 Typer + Rich 在后台日志中的表现。
    """

    rng = random.Random(seed)

    # --- 阶段 1: 欢迎与依赖解析 ---
    rprint("[bold cyan]🚀 HyperInstall CLI[/bold cyan] [dim]v2.0.1[/dim]")
    rprint(f"[green]➜[/green] 正在准备安装 [bold yellow]{package_name}[/bold yellow]...")

    with console.status(
        "[bold green]正在解析依赖树 (Resolving dependency tree)...[/bold green]",
        spinner="dots",
    ):
        scaled_sleep(4.5, delay_scale)
        rprint("[bold green]✔ 依赖解析完成 (Dependency tree resolved)[/bold green]")

    print()

    # --- 阶段 2: 模拟并发下载/构建 ---
    with Progress(
        TextColumn("[bold blue]{task.description}", justify="right"),
        SpinnerColumn(),
        BarColumn(bar_width=None),
        "[progress.percentage]{task.percentage:>3.0f}%",
        "•",
        DownloadColumn(),
        "•",
        TransferSpeedColumn(),
        "•",
        TimeRemainingColumn(),
        console=console,
    ) as progress:
        task1 = progress.add_task("[cyan]下载 Core 核心库", total=1024 * 1024 * 5)
        task2 = progress.add_task("[magenta]编译 Assets 资源", total=1024 * 1024 * 2)
        task3 = progress.add_task("[yellow]安装 Binaries 二进制", total=1024 * 1024 * 8)

        tasks = [task1, task2, task3]

        while not progress.finished:
            for task_id in tasks:
                scaled_sleep(rng.uniform(0.01, 0.04), delay_scale)
                chunk_size = rng.randint(1024 * 10, 1024 * 100)
                progress.update(task_id, advance=chunk_size)

    # --- 阶段 3: 收尾与统计展示 ---
    rprint("[dim]正在清理缓存...[/dim]")

    table = Table(show_header=True, header_style="bold magenta")
    table.add_column("组件", style="dim")
    table.add_column("状态")
    table.add_column("耗时")
    table.add_row("Core Lib", "[green]Installed[/green]", "1.2s")
    table.add_row("Assets", "[green]Built[/green]", "0.8s")
    table.add_row("Binaries", "[green]Linked[/green]", "2.1s")

    success_panel = Panel(
        table,
        title="[bold green]安装成功 (Installation Complete)[/bold green]",
        border_style="green",
        expand=False,
    )

    print()
    rprint(success_panel)
    rprint(f"[bold]✨ {package_name}[/bold] 现在可以使用了！\n")


@app.command()
def plain(
    name: str = typer.Argument("sample", help="要写进日志里的样本名称。"),
    lines: int = typer.Option(5, min=1, max=20, help="输出多少行普通日志。"),
):
    """
    只输出最朴素的 line-oriented 文本，作为无噪声基线。
    """

    print(f"[plain] begin name={name} lines={lines}")
    for index in range(1, lines + 1):
        print(f"[plain] step={index}/{lines} name={name}")
    print(f"[plain] end name={name}")


@app.command("carriage-return")
def carriage_return(
    frames: int = typer.Option(12, min=1, max=60, help="连续重绘多少帧。"),
    delay_scale: float = typer.Option(0.2, min=0.0, help="重绘间隔缩放。"),
    clear_line: bool = typer.Option(False, help="是否在每次重绘前发出 clear-line。"),
):
    """
    只用 CR（可选 ANSI clear-line）重绘单行，观察后台日志是否摊平历史帧。
    """

    print("[cr] begin")
    for frame in range(1, frames + 1):
        prefix = f"{ESC}[2K" if clear_line else ""
        message = f"{prefix}\r[cr] frame={frame:02d}/{frames:02d} progress={'#' * frame}"
        emit(message)
        scaled_sleep(0.05, delay_scale)
    emit("\r[cr] final stable line\n")
    print("[cr] end")


@app.command("ansi-style")
def ansi_style():
    """
    直接输出 ANSI SGR 样式，不依赖 Rich，便于测试原始 ESC 是否被保留。
    """

    red = f"{ESC}[31m"
    green = f"{ESC}[32m"
    bold = f"{ESC}[1m"
    reset = f"{ESC}[0m"
    print("[ansi] begin")
    emit(f"{red}[ansi] red text{reset}\n")
    emit(f"{green}{bold}[ansi] bold green text{reset}\n")
    emit(f"{ESC}[33m[ansi] yellow warning{reset}\n")
    print("[ansi] end")


@app.command("cursor-motion")
def cursor_motion(delay_scale: float = typer.Option(0.2, min=0.0, help="动画间隔缩放。")):
    """
    输出多行，再用 cursor-up + clear-line 重绘，测试简单的多行终端语义。
    """

    print("[cursor] begin")
    print("job A: queued")
    print("job B: queued")
    print("job C: queued")
    scaled_sleep(0.05, delay_scale)

    emit(f"{ESC}[3A")
    emit(f"{ESC}[2Kjob A: running\n")
    emit(f"{ESC}[2Kjob B: running\n")
    emit(f"{ESC}[2Kjob C: queued\n")
    scaled_sleep(0.05, delay_scale)

    emit(f"{ESC}[3A")
    emit(f"{ESC}[2Kjob A: done\n")
    emit(f"{ESC}[2Kjob B: done\n")
    emit(f"{ESC}[2Kjob C: done\n")
    print("[cursor] end")


@app.command("osc-demo")
def osc_demo():
    """
    输出 OSC 标题与超链接序列，观察日志是否原样保留或被上游吞掉。
    """

    title_sequence = f"{OSC}0;DocuTouch Demo Title\x07"
    hyperlink = f"{OSC}8;;https://example.com\x07click-me{OSC}8;;\x07"
    print("[osc] begin")
    emit(title_sequence)
    emit(f"[osc] title sequence emitted\n")
    emit(f"[osc] hyperlink={hyperlink}\n")
    print("[osc] end")


@app.command("alt-screen")
def alt_screen(delay_scale: float = typer.Option(0.2, min=0.0, help="动画间隔缩放。")):
    """
    进入 alt-screen，渲染几帧，再退出，测试这类终端语义在后台日志里的残留形态。
    """

    print("[alt] begin")
    emit(f"{ESC}[?1049h")
    emit(f"{ESC}[2J{ESC}[H")
    emit("ALT SCREEN TITLE\n")
    emit("row-1 pending\n")
    emit("row-2 pending\n")
    scaled_sleep(0.05, delay_scale)
    emit(f"{ESC}[2A")
    emit(f"{ESC}[2Krow-1 done\n")
    emit(f"{ESC}[2Krow-2 done\n")
    scaled_sleep(0.05, delay_scale)
    emit(f"{ESC}[?1049l")
    print("[alt] end")


@app.command("all-samples")
def all_samples():
    """
    串行跑一组紧凑样本，适合一次性比较多种输出类别。
    """

    plain(name="matrix", lines=3)
    carriage_return(frames=5, delay_scale=0.0, clear_line=False)
    ansi_style()
    cursor_motion(delay_scale=0.0)
    osc_demo()


@app.command()
def typewriter(
    text: str = typer.Argument(
        "Hello! This is a slow typewriter demo running in Pueue background. Watch the log grow line by line as you poll it.",
        help="要逐字输出的文本内容。",
    ),
    char_delay: float = typer.Option(0.08, min=0.0, help="每个字符之间的停顿（秒）。"),
    line_width: int = typer.Option(40, min=10, help="每行最大字符数，超过则折行。"),
):
    """
    慢速打字机模式 —— 按字符逐个输出，支持 AI 分段轮询日志。

    适合演示：AI 后台启动长任务后，每隔几秒 read pueue-log:<id>，
    观察日志随时间递增的"流式"体验。
    """
    sys.stdout.reconfigure(line_buffering=True)

    words = text.split()
    current_line = ""
    line_num = 0

    console.print("[bold cyan]── Typewriter Start ──[/bold cyan]")

    for word in words:
        candidate = (current_line + " " + word).strip()
        if len(candidate) > line_width and current_line:
            line_num += 1
            sys.stdout.write(f"\n[{line_num:03d}] ")
            sys.stdout.flush()
            current_line = word
        else:
            current_line = candidate

        for char in word + " ":
            sys.stdout.write(char)
            sys.stdout.flush()
            time.sleep(char_delay)

        if len(current_line) > line_width:
            sys.stdout.write("\n")
            sys.stdout.flush()
            line_num += 1
            current_line = ""

    if current_line.strip():
        sys.stdout.write("\n")
        sys.stdout.flush()
        line_num += 1

    console.print(f"[bold cyan]── Typewriter Done ({line_num} lines) ──[/bold cyan]")


@app.command()
def repl(
    prompt: str = typer.Option("> ", help="每行的提示符。"),
    echo_style: str = typer.Option("green", help="Rich 颜色名，用于渲染回显文字。"),
):
    """
    交互式 REPL 模式 —— 演示 pueue send 的工作效果。

    循环读取 stdin，直到收到 'quit' 或 EOF。

    在 Pueue 后台运行时，可用：
        pueue send <task_id> "your message\\n"
    来向本进程注入输入。
    """
    sys.stdout.reconfigure(line_buffering=True)

    is_bg = not sys.stdin.isatty()
    console.print(
        Panel(
            "[bold cyan]REPL Demo[/bold cyan]\n"
            + ("[dim]后台模式：用 [bold]pueue send <id> 'text\\\\n'[/bold] 发送输入[/dim]"
               if is_bg
               else "[dim]前台模式：直接键入文字，按 Enter 确认，输入 quit 退出[/dim]"),
            border_style="cyan",
            expand=False,
        )
    )

    session: list[tuple[str, str]] = []
    try:
        while True:
            # 前台模式打印提示符；后台（pueue）无 tty，不打印，避免日志噪声
            if not is_bg:
                console.print(f"[dim]{prompt}[/dim]", end="")

            line = input()
            cmd = line.strip()

            if cmd.lower() in ("quit", "exit", "q"):
                break

            if cmd == "":
                continue

            # 简单的内置命令演示
            if cmd == "help":
                console.print(
                    "[bold]内置命令：[/bold] help | history | clear | quit"
                )
                continue

            if cmd == "history":
                if not session:
                    console.print("[dim]（暂无历史）[/dim]")
                else:
                    table = Table(show_header=True, header_style="bold blue")
                    table.add_column("#", style="dim", width=4)
                    table.add_column("输入")
                    table.add_column("回显")
                    for i, (inp, out) in enumerate(session, 1):
                        table.add_row(str(i), inp, out)
                    console.print(table)
                continue

            if cmd == "clear":
                console.clear()
                continue

            # 默认：回显并记录
            echo = f"echo: {cmd}"
            console.print(f"[{echo_style}]{echo}[/{echo_style}]")
            session.append((cmd, echo))

    except EOFError:
        # pueue 关闭 stdin 时正常退出
        pass

    console.print(
        Panel(
            f"[bold]会话结束[/bold]，共处理 [cyan]{len(session)}[/cyan] 条指令。",
            border_style="yellow",
            expand=False,
        )
    )


if __name__ == "__main__":
    app()
