"""Record one terminal take of the M2 run-2 script on the released CLI.

Prepares a fresh clone with the two agent branches from the M2 record, opens a
dedicated console window running driver.sh, and records that window with
ffmpeg's Windows Graphics Capture source (works while the window is covered).
Usage: python -I take.py <take-dir> <m2-run2-repo>
"""
import json
import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASH = r"C:\Program Files\Git\bin\bash.exe"
BASE, CLAUDE, CODEX = "62b9ce4", "2534fe2", "ec1701b"  # M2 run 2: base, Claude Code, Codex


def git(*args, cwd):
    subprocess.run(["git", *args], cwd=cwd, check=True)


def wait_for(path, seconds):
    end = time.time() + seconds
    while not path.exists():
        if time.time() > end:
            raise TimeoutError(path)
        time.sleep(0.2)


take, source = Path(sys.argv[1]), Path(sys.argv[2])
take.mkdir(parents=True)
repo = take / "mri"
git("clone", "-q", "--no-checkout", str(source), str(repo), cwd=take)
# Before checkout: a global autocrlf=true would leave CRLF files that read as modified.
git("config", "core.autocrlf", "false", cwd=repo)
git("checkout", "-q", "-B", "master", BASE, cwd=repo)
heads = subprocess.run(["git", "for-each-ref", "--format=%(refname:short)", "refs/heads"], cwd=repo, check=True, capture_output=True, text=True).stdout.split()
for head in heads:
    if head != "master":
        git("branch", "-q", "-D", head, cwd=repo)
git("branch", "-q", "claude/rename-helper", CLAUDE, cwd=repo)
git("branch", "-q", "codex/negated-flags", CODEX, cwd=repo)
git("remote", "remove", "origin", cwd=repo)
assert not subprocess.run(["git", "status", "--porcelain"], cwd=repo, check=True, capture_output=True, text=True).stdout, "take repo is not clean"

# ShellExecute gives the console its own window; a child process would share ours.
os.startfile("conhost.exe", arguments=f'"{BASH}" --noprofile --norc "{(HERE / "driver.sh").as_posix()}" "{take.as_posix()}" "{HERE.as_posix()}"')
wait_for(take / "hwnd.txt", 60)
hwnd, *layout = (take / "hwnd.txt").read_text().split()
time.sleep(1)

capture = take / "capture.mkv"
started = time.time()
ffmpeg = subprocess.Popen(
    ["ffmpeg", "-hide_banner", "-loglevel", "error", "-f", "lavfi",
     "-i", f"gfxcapture=hwnd={hwnd}:max_framerate=30:capture_cursor=0,hwdownload,format=bgra",
     "-fps_mode", "cfr", "-r", "30", "-c:v", "libx264", "-preset", "ultrafast", "-qp", "0", "-pix_fmt", "yuv444p",
     str(capture)],
    stdin=subprocess.PIPE,
)
wait_for(capture, 30)
time.sleep(2)
go = time.time()
(take / "go").touch()
wait_for(take / "done", 20 * 60)
ffmpeg.communicate(b"q", timeout=60)
(take / "close").touch()
(take / "take.json").write_text(json.dumps({
    "capture": capture.name, "ffmpeg_started": started, "go": go, "window": layout,
    "branches": {"base": BASE, "claude/rename-helper": CLAUDE, "codex/negated-flags": CODEX, "source": str(source)},
}, indent=2))
print("take done", take, "ffmpeg exit", ffmpeg.returncode)
