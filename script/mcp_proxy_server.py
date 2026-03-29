"""
用于修复部分下游 MCP 识别问题

"""

import sys
import subprocess
import os
import json
import datetime
import threading
import time
import signal

# =========================================================
# 配置区
# =========================================================
REAL_SERVER_EXE = r"D:\MyFile\Code\experiment_1\micheng\docutouch\target\release\docutouch.exe"
LOG_FILE = r"D:\MyFile\Code\experiment_1\micheng\docutouch\script\temporary\debug_traffic.log"

HEARTBEAT_INTERVAL = 5.0
HEARTBEAT_ID = "internal-heartbeat-check"
# =========================================================

stdin_lock = threading.Lock()
shutdown_event = threading.Event()


def log_to_file(msg, level="INFO"):
    """写入日志文件，带毫秒级时间戳"""
    try:
        timestamp = datetime.datetime.now().strftime("%H:%M:%S.%f")[:-3]
        log_line = f"[{timestamp}][{level}] {msg}\n"
        with open(LOG_FILE, "a", encoding="utf-8") as f:
            f.write(log_line)
            f.flush()
    except Exception:
        pass


def safe_terminate_process(process, wait_timeout=3):
    """优雅关闭子进程，必要时强杀"""
    if process.poll() is not None:
        return

    try:
        log_to_file("Terminating child process...", "SYSTEM")
        process.terminate()
        process.wait(timeout=wait_timeout)
        log_to_file("Child process terminated gracefully.", "SYSTEM")
    except subprocess.TimeoutExpired:
        log_to_file("Terminate timeout, killing child process...", "SYSTEM")
        try:
            process.kill()
            process.wait(timeout=2)
            log_to_file("Child process killed.", "SYSTEM")
        except Exception as e:
            log_to_file(f"Failed to kill child process: {e}", "ERROR")
    except Exception as e:
        log_to_file(f"Terminate process error: {e}", "ERROR")


def close_child_stdin(process):
    """关闭子进程 stdin，提示它不再有上游输入"""
    try:
        if process.stdin:
            process.stdin.close()
    except Exception:
        pass


def read_input_loop(process):
    """
    线程1：读取上游 stdin，并转发给子进程
    注意：不用 daemon，退出时由主线程控制
    """
    log_to_file("Input Forwarder Thread Started", "SYSTEM")

    try:
        while not shutdown_event.is_set():
            chunk = sys.stdin.buffer.readline()

            if not chunk:
                log_to_file("Client stdin closed.", "SYSTEM")
                shutdown_event.set()
                close_child_stdin(process)
                break

            with stdin_lock:
                if process.poll() is None and process.stdin:
                    process.stdin.write(chunk)
                    process.stdin.flush()
                else:
                    log_to_file("Child process already exited while forwarding input.", "WARN")
                    shutdown_event.set()
                    break

    except Exception as e:
        log_to_file(f"Input Loop Error: {e}", "ERROR")
        shutdown_event.set()


def heartbeat_loop(process):
    """
    线程2：定时向子进程发送 ping
    """
    log_to_file("Heartbeat Injector Started", "SYSTEM")
    time.sleep(2)

    while not shutdown_event.is_set():
        if process.poll() is not None:
            log_to_file("Child process exited, heartbeat loop stopping.", "SYSTEM")
            shutdown_event.set()
            break

        try:
            ping_payload = {
                "jsonrpc": "2.0",
                "method": "ping",
                "id": HEARTBEAT_ID
            }
            ping_bytes = json.dumps(ping_payload, ensure_ascii=False).encode("utf-8") + b"\n"

            with stdin_lock:
                if process.poll() is None and process.stdin:
                    process.stdin.write(ping_bytes)
                    process.stdin.flush()
                    log_to_file("Ping sent.", "HEARTBEAT")
                else:
                    shutdown_event.set()
                    break

        except Exception as e:
            log_to_file(f"Heartbeat Failed: {e}", "ERROR")
            shutdown_event.set()
            break

        shutdown_event.wait(HEARTBEAT_INTERVAL)


def stderr_reader_loop(process):
    """
    线程3：把子进程 stderr 写到日志
    """
    log_to_file("Stderr Reader Thread Started", "SYSTEM")

    try:
        while not shutdown_event.is_set():
            line = process.stderr.readline()
            if not line:
                break

            text = line.decode("utf-8", errors="replace").rstrip()
            if text:
                log_to_file(f"[STDERR] {text}", "REMOTE_LOG")

    except Exception as e:
        log_to_file(f"Stderr Reader Error: {e}", "ERROR")
    finally:
        log_to_file("Stderr Reader Thread Exited", "SYSTEM")


def setup_binary_stdio_for_windows():
    if sys.platform == "win32":
        try:
            import msvcrt
            msvcrt.setmode(sys.stdin.fileno(), os.O_BINARY)
            msvcrt.setmode(sys.stdout.fileno(), os.O_BINARY)
        except Exception as e:
            log_to_file(f"setmode failed: {e}", "WARN")


def install_signal_handlers():
    def _handle_signal(signum, frame):
        log_to_file(f"Received signal: {signum}", "SYSTEM")
        shutdown_event.set()

    try:
        signal.signal(signal.SIGINT, _handle_signal)
    except Exception:
        pass

    try:
        signal.signal(signal.SIGTERM, _handle_signal)
    except Exception:
        pass


def run_icu_proxy():
    install_signal_handlers()

    env = os.environ.copy()
    env["PYTHONUNBUFFERED"] = "1"
    env["PYTHONIOENCODING"] = "utf-8"

    os.makedirs(os.path.dirname(LOG_FILE), exist_ok=True)
    with open(LOG_FILE, "w", encoding="utf-8") as f:
        f.write(f"=== MCP ICU Sentinel Started at {datetime.datetime.now()} ===\n")

    if not os.path.exists(REAL_SERVER_EXE):
        log_to_file(f"Rust server not found: {REAL_SERVER_EXE}", "ERROR")
        raise FileNotFoundError(REAL_SERVER_EXE)

    setup_binary_stdio_for_windows()

    log_to_file(f"Starting Rust server: {REAL_SERVER_EXE}", "SYSTEM")

    process = subprocess.Popen(
        [REAL_SERVER_EXE],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
        bufsize=0
    )

    t_input = threading.Thread(target=read_input_loop, args=(process,), name="input-forwarder")
    t_hb = threading.Thread(target=heartbeat_loop, args=(process,), name="heartbeat")
    t_err = threading.Thread(target=stderr_reader_loop, args=(process,), name="stderr-reader")

    t_input.start()
    t_hb.start()
    t_err.start()

    try:
        while not shutdown_event.is_set():
            line = process.stdout.readline()

            if not line:
                log_to_file("Process stdout closed. Main loop exiting.", "SYSTEM")
                shutdown_event.set()
                break

            stripped = line.strip()
            if not stripped:
                continue

            if stripped.startswith(b"{"):
                try:
                    data = json.loads(stripped.decode("utf-8", errors="replace"))
                    msg_id = data.get("id")

                    if msg_id == HEARTBEAT_ID:
                        log_to_file("Pong received! Server is ALIVE.", "HEARTBEAT")
                        continue

                    clean_bytes = json.dumps(data, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
                    sys.stdout.buffer.write(clean_bytes + b"\n")
                    sys.stdout.buffer.flush()

                    log_to_file(f"Forwarded Response ID: {msg_id}", "TRAFFIC")

                except json.JSONDecodeError:
                    preview = stripped[:200].decode("utf-8", errors="replace")
                    log_to_file(f"JSON Parse Error: {preview}", "WARN")
            else:
                preview = stripped[:200].decode("utf-8", errors="replace")
                log_to_file(f"Dropped Junk: {preview}", "FILTER")

            if process.poll() is not None:
                log_to_file(f"Child process exited with code: {process.returncode}", "SYSTEM")
                shutdown_event.set()
                break

    except KeyboardInterrupt:
        log_to_file("KeyboardInterrupt received.", "SYSTEM")
        shutdown_event.set()

    except Exception as e:
        log_to_file(f"Main Loop Error: {e}", "ERROR")
        shutdown_event.set()

    finally:
        close_child_stdin(process)
        safe_terminate_process(process)

        t_hb.join(timeout=2)
        t_err.join(timeout=2)
        # t_input 可能阻塞在 readline；不给它 daemon，但也不无限等
        t_input.join(timeout=1)

        log_to_file("Proxy shutdown complete.", "SYSTEM")


if __name__ == "__main__":
    run_icu_proxy()