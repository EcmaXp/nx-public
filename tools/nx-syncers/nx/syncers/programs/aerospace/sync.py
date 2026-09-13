import subprocess


def sync_aerospace() -> bool:
    subprocess.check_call(["aerospace", "reload-config"])
    return True
