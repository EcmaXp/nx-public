from datetime import datetime


def now(dt: datetime | None = None):
    if not dt:
        dt = datetime.now()

    return dt.isoformat(" ", "seconds")
