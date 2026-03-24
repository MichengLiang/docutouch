import asyncio
import datetime
import json
import shutil
import textwrap
from pathlib import Path
from typing import Any

from fastmcp import Client
from fastmcp.client.transports import StdioTransport


REPO_ROOT = Path(__file__).resolve().parent.parent
DEBUG_EXE = REPO_ROOT / "target" / "debug" / "docutouch.exe"
RELEASE_EXE = REPO_ROOT / "target" / "release" / "docutouch.exe"
SERVER_EXE = DEBUG_EXE if DEBUG_EXE.exists() else RELEASE_EXE
WORKSPACE = REPO_ROOT / "tmp" / "example_workspace"
DEMO_DIR_NAME = "__diagnostics_demo__"
DEMO_DIR = WORKSPACE / DEMO_DIR_NAME
STDERR_LOG = WORKSPACE / "diagnostics_demo_stderr.log"


class UI:
    HEADER = "\033[95m"
    BLUE = "\033[94m"
    CYAN = "\033[96m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    BOLD = "\033[1m"
    RESET = "\033[0m"

    @staticmethod
    def section(title: str):
        print(f"\n{UI.HEADER}{'=' * 76}")
        print(f" {title}")
        print(f"{'=' * 76}{UI.RESET}")

    @staticmethod
    def ok(msg: str):
        print(f"{UI.GREEN}[OK] {msg}{UI.RESET}")

    @staticmethod
    def warn(msg: str):
        print(f"{UI.YELLOW}[WARN] {msg}{UI.RESET}")

    @staticmethod
    def err(msg: str):
        print(f"{UI.RED}[ERR] {msg}{UI.RESET}")

    @staticmethod
    def info(key: str, value: str):
        print(f"{UI.BLUE}{key:<20}{UI.RESET}: {value}")

    @staticmethod
    def code_block(title: str, text: str):
        print(f"\n{UI.CYAN}--- {title} ---{UI.RESET}")
        print(text)
        print(f"{UI.CYAN}{'-' * 40}{UI.RESET}")


def force_to_dict(obj: Any) -> Any:
    if isinstance(obj, (str, int, float, bool, type(None))):
        return obj
    if isinstance(obj, list):
        return [force_to_dict(i) for i in obj]
    if isinstance(obj, dict):
        return {k: force_to_dict(v) for k, v in obj.items()}
    if hasattr(obj, "model_dump"):
        return obj.model_dump(exclude_none=True)
    if hasattr(obj, "dict") and callable(obj.dict):
        return obj.dict(exclude_none=True)
    if hasattr(obj, "__dict__"):
        return force_to_dict(obj.__dict__)
    return str(obj)


def extract_text_content(result: Any) -> str:
    parts = []
    content = getattr(result, "content", None) or []
    for block in content:
        if hasattr(block, "text"):
            parts.append(block.text)
        else:
            parts.append(str(block))
    return "\n".join(parts).strip()


def pretty_print_tool_result(tool_name: str, result: Any, max_text_chars: int = 6000):
    print(f"\n{UI.CYAN}>>> 工具返回: {tool_name}{UI.RESET}")
    text = extract_text_content(result)
    if text:
        if len(text) > max_text_chars:
            text = text[:max_text_chars] + "\n...[truncated for terminal]"
        UI.code_block(f"{tool_name} / text", text)

    data = getattr(result, "data", None)
    if data is not None:
        try:
            clean = force_to_dict(data)
            dumped = json.dumps(clean, ensure_ascii=False, indent=2)
            if len(dumped) > 2000:
                dumped = dumped[:2000] + "\n...[truncated for terminal]"
            UI.code_block(f"{tool_name} / data", dumped)
        except Exception as exc:
            UI.warn(f"结构化数据打印失败: {exc}")


def pretty_print_tool_error(tool_name: str, exc: Exception):
    UI.err(f"{tool_name} 返回错误")
    UI.code_block(f"{tool_name} / error", str(exc))


async def invoke_tool(
    client: Client,
    name: str,
    arguments: dict,
    *,
    expect_error: bool = False,
    max_text_chars: int = 6000,
):
    UI.section(f"调用工具: {name}")
    UI.code_block("arguments", json.dumps(arguments, ensure_ascii=False, indent=2))

    try:
        result = await client.call_tool(name=name, arguments=arguments)
    except Exception as exc:
        pretty_print_tool_error(name, exc)
        if not expect_error:
            raise
        return None

    if expect_error:
        UI.warn("本次原本预期失败，但调用却成功了")
    else:
        UI.ok("调用成功")
    pretty_print_tool_result(name, result, max_text_chars=max_text_chars)
    return result


def build_success_patch() -> str:
    return textwrap.dedent(
        f"""\
        *** Begin Patch
        *** Add File: {DEMO_DIR_NAME}/created/success.txt
        +Hello from the diagnostics showcase.
        *** End Patch
        """
    )


def build_invalid_add_patch() -> str:
    return textwrap.dedent(
        f"""\
        *** Begin Patch
        *** Add File: {DEMO_DIR_NAME}/broken.txt
        broken line
        *** End Patch
        """
    )


def build_missing_target_patch() -> str:
    return textwrap.dedent(
        f"""\
        *** Begin Patch
        *** Update File: {DEMO_DIR_NAME}/missing.txt
        @@
        -old
        +new
        *** End Patch
        """
    )


def build_context_mismatch_patch() -> str:
    return textwrap.dedent(
        f"""\
        *** Begin Patch
        *** Update File: {DEMO_DIR_NAME}/src/app.py
        @@
        -missing = 1
        +value = 2
        *** End Patch
        """
    )


def build_target_anchor_mismatch_patch() -> str:
    return textwrap.dedent(
        f"""\
        *** Begin Patch
        *** Update File: {DEMO_DIR_NAME}/src/handler.py
        @@ def handler():
        -    missing = 1
        +    value = 2
        *** End Patch
        """
    )


def build_partial_failure_patch() -> str:
    return textwrap.dedent(
        f"""\
        *** Begin Patch
        *** Add File: {DEMO_DIR_NAME}/partial/created.txt
        +hello
        *** Update File: {DEMO_DIR_NAME}/partial/missing.txt
        @@
        -old
        +new
        *** End Patch
        """
    )


def build_large_partial_failure_patch() -> str:
    body = ["*** Begin Patch"]
    for index in range(10):
        body.append(f"*** Add File: {DEMO_DIR_NAME}/large/created-{index}.txt")
        body.append(f"+hello {index}")
    body.extend(
        [
            f"*** Update File: {DEMO_DIR_NAME}/large/missing.txt",
            "@@",
            "-old",
            "+new",
            "*** End Patch",
        ]
    )
    return "\n".join(body) + "\n"


def build_move_write_failure_patch() -> str:
    return textwrap.dedent(
        f"""\
        *** Begin Patch
        *** Update File: {DEMO_DIR_NAME}/move/src/name.txt
        *** Move to: {DEMO_DIR_NAME}/move/blocked/dir/name.txt
        @@
        -from
        +new
        *** End Patch
        """
    )


def local_prepare():
    WORKSPACE.mkdir(parents=True, exist_ok=True)
    if DEMO_DIR.exists():
        shutil.rmtree(DEMO_DIR, ignore_errors=True)

    (DEMO_DIR / "src").mkdir(parents=True, exist_ok=True)
    (DEMO_DIR / "move" / "src").mkdir(parents=True, exist_ok=True)
    (DEMO_DIR / "partial").mkdir(parents=True, exist_ok=True)
    (DEMO_DIR / "large").mkdir(parents=True, exist_ok=True)

    (DEMO_DIR / "src" / "app.py").write_text("value = 1\n", encoding="utf-8")
    (DEMO_DIR / "src" / "handler.py").write_text(
        "def handler():\n    value = 1\n", encoding="utf-8"
    )
    (DEMO_DIR / "move" / "src" / "name.txt").write_text("from\n", encoding="utf-8")
    (DEMO_DIR / "move" / "blocked").write_text("not a directory\n", encoding="utf-8")

    UI.ok("演示工作区已准备完成")


def local_cleanup():
    if DEMO_DIR.exists():
        shutil.rmtree(DEMO_DIR, ignore_errors=True)
        UI.ok("演示工作区已清理")
    else:
        UI.ok("演示工作区已为空，无需清理")


async def showcase_apply_patch(client: Client):
    cases = [
        (
            "Success / Full Success",
            {"patch": build_success_patch()},
            False,
        ),
        (
            "Failure / Empty Patch",
            {"patch": ""},
            True,
        ),
        (
            "Failure / Outer Invalid Add Line",
            {"patch": build_invalid_add_patch()},
            True,
        ),
        (
            "Failure / Update Target Missing",
            {"patch": build_missing_target_patch()},
            True,
        ),
        (
            "Failure / Context Mismatch",
            {"patch": build_context_mismatch_patch()},
            True,
        ),
        (
            "Failure / Target Anchor Mismatch",
            {"patch": build_target_anchor_mismatch_patch()},
            True,
        ),
        (
            "Failure / Partial Success",
            {"patch": build_partial_failure_patch()},
            True,
        ),
        (
            "Failure / Large Partial Success",
            {"patch": build_large_partial_failure_patch()},
            True,
        ),
        (
            "Failure / Move Write Error",
            {"patch": build_move_write_failure_patch()},
            True,
        ),
    ]

    for title, arguments, expect_error in cases:
        UI.section(title)
        await invoke_tool(
            client,
            "apply_patch",
            arguments,
            expect_error=expect_error,
            max_text_chars=12000,
        )


async def main():
    UI.section("DocuTouch Diagnostics Showcase")
    UI.info("Repo Root", str(REPO_ROOT))
    UI.info("Server EXE", str(SERVER_EXE))
    UI.info("Workspace", str(WORKSPACE))
    UI.info("Demo Dir", str(DEMO_DIR))
    UI.info("Stderr Log", str(STDERR_LOG))

    if not SERVER_EXE.exists():
        UI.err("没有找到 docutouch.exe，请先编译 target/release 或 target/debug")
        return

    local_prepare()

    transport = StdioTransport(
        command=str(SERVER_EXE),
        args=[],
        keep_alive=False,
        log_file=STDERR_LOG,
    )
    client = Client(
        transport=transport,
        name="DocuTouch Diagnostics Showcase",
        timeout=datetime.timedelta(seconds=60),
    )

    try:
        async with client:
            UI.ok("已连接到 MCP 服务端")

            UI.section("列出工具")
            tools = await client.list_tools()
            UI.info("Tool Count", str(len(tools)))
            for tool in tools:
                print(f"- {tool.name}")

            await invoke_tool(client, "set_workspace", {"path": str(WORKSPACE)})

            await invoke_tool(
                client,
                "list_directory",
                {
                    "relative_path": ".",
                    "max_depth": 3,
                    "show_hidden": True,
                    "include_gitignored": False,
                },
                max_text_chars=8000,
            )

            await invoke_tool(
                client,
                "read_file",
                {
                    "relative_path": f"{DEMO_DIR_NAME}/src/handler.py",
                    "show_line_numbers": True,
                },
                max_text_chars=4000,
            )

            await showcase_apply_patch(client)

            await invoke_tool(
                client,
                "list_directory",
                {
                    "relative_path": DEMO_DIR_NAME,
                    "max_depth": 4,
                    "show_hidden": True,
                    "include_gitignored": False,
                },
                max_text_chars=10000,
            )
    except Exception as exc:
        UI.err(f"主流程失败: {exc}")
        UI.warn("可以查看 stderr 日志进一步排查")
    finally:
        local_cleanup()
        UI.section("Showcase Finished")
        UI.info("Workspace", str(WORKSPACE))
        UI.info("Stderr Log", str(STDERR_LOG))


if __name__ == "__main__":
    asyncio.run(main())
