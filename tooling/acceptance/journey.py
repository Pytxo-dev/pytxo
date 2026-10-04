# Drives an installed Pytxo Desktop through Windows UI Automation, with the real
# OS pointer and keyboard, for cloud acceptance (see run-acceptance.ps1).
#
#   python journey.py --out <dir> --mode full --repo <fixture> --mission <file> [--team "Claude Code,…"]
#   python journey.py --out <dir> --mode layout
#   python journey.py --out <dir> --mode probe --name <label> [--onboard]
#
# full:   onboarding -> real folder dialog -> mixed-agent plan -> run -> Review ->
#         stale refusal -> refresh -> Apply, with the screen recorded by ffmpeg.
# layout: guided-example onboarding, then each destination checked for controls
#         that fall outside the window or a horizontally scrolling page.
# probe:  what Desktop shows (for the upgrade check), optionally onboarding first.
# WebView2 150+ no longer serves remote debugging to this app, so nothing here
# uses CDP. Outcomes come from Pytxo's own ledger and the project files.
import argparse
import ctypes
import ctypes.wintypes
import json
import re
import sqlite3
import subprocess
import time
import traceback
from pathlib import Path

from PIL import ImageGrab
from pywinauto import Desktop, mouse
from pywinauto.keyboard import send_keys

parser = argparse.ArgumentParser()
parser.add_argument("--out", required=True)
parser.add_argument("--mode", choices=["full", "layout", "probe"], required=True)
parser.add_argument("--repo")
parser.add_argument("--mission")
parser.add_argument("--team", default="Claude Code,Cursor Agent,OpenCode,Antigravity")
parser.add_argument("--name", default="probe")
parser.add_argument("--onboard", action="store_true")
args = parser.parse_args()

out = Path(args.out)
out.mkdir(parents=True, exist_ok=True)
now_ms = lambda: round(time.time() * 1000)
receipt = {"mode": args.mode, "started": now_ms(), "steps": [], "checks": {}, "pointer": [], "marks": {}}


def step(label, **extra):
    receipt["steps"].append({"t": now_ms(), "label": label, **extra})
    print(f"{(now_ms() - receipt['started']) / 1000:.1f}s {label}", flush=True)


def mark(name):
    receipt["marks"][name] = now_ms()


def save():
    (out / ("receipt.json" if args.mode != "probe" else f"{args.name}.json")).write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")


def shot(name):
    ImageGrab.grab().save(out / f"{name}.png")


win = Desktop(backend="uia").window(title="Pytxo Desktop", control_type="Window")
win.wait("visible", timeout=120)
win.set_focus()
win.maximize()
time.sleep(2)


def spec(title=None, title_re=None, control_type=None, index=0):
    criteria = {"found_index": index}
    if title is not None:
        criteria["title"] = title
    if title_re is not None:
        criteria["title_re"] = title_re
    if control_type is not None:
        criteria["control_type"] = control_type
    return win.child_window(**criteria)


def wait(target, timeout=60, state="exists"):
    return target.wait(state, timeout=timeout)


def present(target, timeout=0.5):
    try:
        target.wait("exists", timeout=timeout)
        return True
    except Exception:
        return False


def cursor():
    point = ctypes.wintypes.POINT()
    ctypes.windll.user32.GetCursorPos(ctypes.byref(point))
    return point.x, point.y


def glide(x, y, seconds=0.5):
    """Moves the real pointer to (x, y) along an eased path, as a person would."""
    x0, y0 = cursor()
    steps = max(10, int(seconds * 90))
    for index in range(1, steps + 1):
        t = index / steps
        eased = t * t * (3 - 2 * t)
        mouse.move(coords=(round(x0 + (x - x0) * eased), round(y0 + (y - y0) * eased)))
        time.sleep(seconds / steps)


def press(target, label, timeout=60, settle=0.4, surface="desktop"):
    element = target.wait("visible enabled", timeout=timeout)
    try:
        element.iface_scroll_item.ScrollIntoView()
        time.sleep(0.3)
    except Exception:
        pass
    middle = element.rectangle().mid_point()
    receipt["pointer"].append({"t": now_ms(), "kind": "hover", "label": label, "x": middle.x, "y": middle.y, "surface": surface})
    glide(middle.x, middle.y)
    time.sleep(settle)
    receipt["pointer"].append({"t": now_ms(), "kind": "click", "label": label, "surface": surface})
    mouse.click(coords=(middle.x, middle.y))
    return element


def type_text(text, pause=0.008):
    """Types like a keyboard; pywinauto's send_keys syntax characters are escaped."""
    escaped = re.sub(r"([+^%~(){}\[\]])", r"{\1}", text).replace("\n", "{ENTER}")
    send_keys(escaped, with_spaces=True, pause=pause)


def ledger(sql, *params):
    database = Path(args.repo) / ".pytxo" / "data" / "pytxo.db"
    connection = sqlite3.connect(f"file:{database.as_posix()}?mode=ro", uri=True, timeout=10)
    try:
        return connection.execute(sql, params).fetchall()
    finally:
        connection.close()


def latest_run():
    rows = ledger("select id, status, finished_at from runs order by started_at desc limit 1")
    return rows[0] if rows else None


def contract(run_id):
    rows = ledger("select prepared_manifest_json, prepared_digest, apply_status from run_contracts where run_id = ?", run_id)
    if not rows or not rows[0][0]:
        return None
    manifest = json.loads(rows[0][0])
    return {"files": sorted(file["path"] for file in manifest.get("files", [])), "digest": rows[0][1], "apply": rows[0][2]}


def wait_for(predicate, timeout, interval=2.0, what="condition"):
    deadline = time.time() + timeout
    while time.time() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(interval)
    raise TimeoutError(f"Timed out waiting for {what}")


def onboard(example):
    press(spec("Get started", control_type="Button"), "Get started", timeout=120)
    try:
        spec(title_re="Finding your coding agents.*").wait_not("exists", timeout=90)
    except Exception:
        pass
    time.sleep(1)
    shot("onboarding-agents")
    press(spec("Continue", control_type="Button"), "Continue")
    if example:
        press(spec("Try the guided example", control_type="Button"), "Try the guided example")
    else:
        press(spec("Select folder", control_type="Button"), "Select folder")
        # The Windows folder dialog, answered like a person would: click the Folder
        # box, type the path, choose Select Folder. (Desktop runs as another user,
        # so UI Automation can find the controls but not set their values.)
        dialog = Desktop(backend="uia").window(title="Select project workspace")
        dialog.wait("visible", timeout=60)
        press(dialog.child_window(auto_id="1152", found_index=0), "Folder")
        send_keys("^a")
        type_text(str(Path(args.repo).resolve()))
        for _ in range(3):
            if not present(dialog, 1):
                break
            press(dialog.child_window(title="Select Folder", control_type="Button"), "Select Folder")
            time.sleep(2)
        wait(spec(title_re="Workspace selected.*"), 60)
    press(spec("Continue", control_type="Button"), "Continue")
    press(spec("Enter Pytxo Desktop", control_type="Button"), "Enter Pytxo Desktop")
    step("desktop entered")
    time.sleep(2)


def fits(name, controls=()):
    """Named controls must sit fully inside the window, and the page must not scroll sideways."""
    frame = win.rectangle()
    outside = []
    for label, target in controls:
        rect = target.wait("visible", timeout=30).rectangle()
        if rect.left < frame.left - 1 or rect.top < frame.top - 1 or rect.right > frame.right + 1 or rect.bottom > frame.bottom + 1:
            outside.append(label)
    sideways = False
    try:
        sideways = bool(spec(control_type="Document").wrapper_object().iface_scroll.CurrentHorizontallyScrollable)
    except Exception:
        pass
    receipt["checks"].setdefault("fit", {})[name] = {"outside": outside, "sideways": sideways}
    shot(name)
    if outside or sideways:
        raise AssertionError(f"{name} does not fit: outside {outside}, horizontal scroll {sideways}")


def start_recording():
    # imageio-ffmpeg ships a static ffmpeg on PyPI; package-manager installs are flaky on runners.
    import imageio_ffmpeg
    ffmpeg = imageio_ffmpeg.get_ffmpeg_exe()
    target = out / "screen.mkv"
    recorder = subprocess.Popen(
        [ffmpeg, "-hide_banner", "-loglevel", "error", "-use_wallclock_as_timestamps", "1", "-f", "gdigrab", "-framerate", "30", "-draw_mouse", "1", "-i", "desktop",
         "-copyts", "-c:v", "libx264", "-preset", "ultrafast", "-crf", "16", "-pix_fmt", "yuv420p", "-y", str(target)],
        stdin=subprocess.PIPE,
    )
    spawned = now_ms()
    time.sleep(2)

    def stop():
        try:
            recorder.communicate(b"q", timeout=120)
        except Exception:
            recorder.kill()
        first = spawned
        # The recording keeps wall-clock timestamps; its start is the first frame's epoch time.
        info = subprocess.run([ffmpeg, "-hide_banner", "-i", str(target)], capture_output=True, text=True).stderr
        start = re.search(r"start: ([0-9.]+)", info)
        if start:
            first = round(float(start.group(1)) * 1000)
        (out / "screen.json").write_text(json.dumps({"first": first, "spawned": spawned}) + "\n", encoding="utf-8")

    return stop


stop_recording = start_recording() if args.mode == "full" else None
try:
    if args.mode == "probe":
        receipt["onboarding_shown"] = present(spec("Get started", control_type="Button"), 20)
        if args.onboard and receipt["onboarding_shown"]:
            # Releases differ in their setup steps, so finish setup by whichever of
            # these each step offers, until no setup step remains.
            choices = ["Get started", "Skip for now", "Try the guided example", "Continue", "Next", "Finish", "Enter Pytxo Desktop", "Open Pytxo Desktop"]
            idle = 0
            while idle < 3 and len(receipt["pointer"]) < 60:
                found = next((name for name in choices if present(spec(name, control_type="Button"), 2)), None)
                if not found:
                    idle += 1
                    continue
                idle = 0
                press(spec(found, control_type="Button"), found)
                time.sleep(2.5)
            if not present(spec("Get started", control_type="Button"), 2) and not present(spec(title_re="^Setup progress$"), 2):
                step("desktop entered")
        time.sleep(2)
        receipt["texts"] = sorted({element.window_text() for element in win.descendants(control_type="Text") if element.window_text().strip()})[:200]
        shot(args.name)
    elif args.mode == "layout":
        onboard(example=True)
        fits("work", [("New work", spec("New work", control_type="Button", index=0))])
        press(spec("New work", control_type="Button", index=0), "New work")
        wait(spec("What should Pytxo do?", control_type="Edit"), 30)
        fits("new-work", [("Build plan", spec("Build plan", control_type="Button"))])
        press(spec("History", index=0), "History")
        time.sleep(1.5)
        fits("history")
        press(spec("Setup", index=0), "Setup")
        try:
            spec(title_re="Checking installed CLIs.*").wait_not("exists", timeout=90)
        except Exception:
            pass
        time.sleep(1)
        fits("setup")
        step("layout checked")
    else:
        mission = Path(args.mission).read_text(encoding="utf-8").replace("\r", "").strip()
        onboard(example=False)
        press(spec("New work", control_type="Button", index=0), "New work")
        request = press(spec("What should Pytxo do?", control_type="Edit"), "Request")
        mark("typing")
        type_text(mission)
        mark("typed")
        agent = spec("Agent CLI", control_type="ComboBox")
        if "OpenAI Codex" not in agent.wait("visible enabled", timeout=60).window_text():
            press(agent, "Agent CLI")
            type_text("OpenAI Codex")
            send_keys("{ENTER}")
        for name in [entry.strip() for entry in args.team.split(",") if entry.strip()]:
            box = spec(name, control_type="CheckBox")
            if box.wait("visible", timeout=30).get_toggle_state() != 1:
                press(box, name, settle=0.2)
        press(spec("Build plan", control_type="Button"), "Build plan")
        heading = wait(spec(title_re="^(Review plan|Plan blocked|Plan needs verification)$"), 180).window_text()
        mark("plan")
        receipt["checks"]["plan"] = {"heading": heading}
        time.sleep(2.5)
        fits("plan")
        if heading != "Review plan":
            raise AssertionError(f"Plan not ready: {heading}")
        step("plan ready")

        before = latest_run()
        press(spec(title_re="^Run", control_type="Button", index=0), "Run")
        mark("run")
        run = wait_for(lambda: (row := latest_run()) and row != before and row, 120, what="the run to start")
        run_id = run[0]
        step("run", id=run_id)
        run = wait_for(lambda: (row := latest_run()) and row[2] and row, 20 * 60, interval=3, what="the run to finish")
        mark("settled")
        agents = ledger("select task_id, status, exit_code from agents where run_id = ? order by task_id", run_id)
        prepared = wait_for(lambda: contract(run_id), 300, what="the prepared change set")
        receipt["checks"]["fleet"] = {"status": run[1], "workers": len(agents), "completed": sum(1 for _, status, _ in agents if status == "completed"), "agents": agents}
        time.sleep(3)
        shot("fleet")
        step("run settled", **receipt["checks"]["fleet"])

        press(spec("Review changes", control_type="Button", index=0), "Review changes")
        wait(spec("Apply reviewed changes", control_type="Button"), 120)
        mark("review")
        time.sleep(2.5)
        # Read down the first file's changes, as a reviewer would.
        changes = spec(title_re="^Changes in .*", index=0)
        if present(changes, 10):
            middle = changes.wrapper_object().rectangle().mid_point()
            receipt["pointer"].append({"t": now_ms(), "kind": "hover", "label": "Changes", "x": middle.x, "y": middle.y, "surface": "desktop"})
            glide(middle.x, middle.y, 0.7)
            for _ in range(4):
                mouse.scroll(coords=(middle.x, middle.y), wheel_dist=-3)
                time.sleep(0.65)
            mark("read")
        receipt["checks"]["review"] = {"files": prepared["files"], "digest": prepared["digest"]}
        fits("review", [("Apply reviewed changes", spec("Apply reviewed changes", control_type="Button"))])
        step("review", **receipt["checks"]["review"])

        # An unrelated file after review must make Apply refuse, with nothing written.
        note = Path(args.repo) / "operator-note.txt"
        note.write_text("A file added after review.\n", encoding="utf-8")
        mark("note")
        press(spec("Apply reviewed changes", control_type="Button"), "Apply reviewed changes")
        press(spec("Apply exact package", control_type="Button"), "Apply exact package")
        wait_for(lambda: contract(run_id)["apply"] == "stale", 120, what="the stale refusal")
        mark("stale")
        time.sleep(3)
        shot("stale")
        receipt["checks"]["stale"] = {"apply": contract(run_id)["apply"]}
        step("stale refused", **receipt["checks"]["stale"])
        note.unlink()

        press(spec("Refresh review", control_type="Button"), "Refresh review")
        refreshed = wait_for(lambda: (state := contract(run_id))["apply"] == "ready" and state, 600, what="the refreshed review")
        receipt["checks"]["refreshed"] = {"files": refreshed["files"], "digest": refreshed["digest"]}
        mark("refreshed")
        time.sleep(1.5)
        press(spec("Apply reviewed changes", control_type="Button"), "Apply reviewed changes")
        press(spec("Apply exact package", control_type="Button"), "Apply exact package")
        applied = wait_for(lambda: (state := contract(run_id)) and state["apply"] not in ("ready", "applying") and state, 300, what="Apply to finish")
        mark("applied")
        time.sleep(3)
        receipt["checks"]["apply"] = {"status": applied["apply"]}
        shot("applied")
        step("apply", **receipt["checks"]["apply"])
        if applied["apply"] != "applied":
            raise AssertionError(f"Apply did not succeed: {applied['apply']}")

        press(spec("History", index=0), "History")
        time.sleep(2)
        shot("history")
    receipt["result"] = "passed"
except Exception as error:
    receipt["result"] = "failed"
    receipt["error"] = "".join(traceback.format_exception(error))
    try:
        shot("failure")
        (out / "failure-texts.txt").write_text("\n".join(element.window_text() for element in win.descendants() if element.window_text().strip()), encoding="utf-8")
    except Exception:
        pass
    raise
finally:
    if stop_recording:
        stop_recording()
    receipt["finished"] = now_ms()
    save()
