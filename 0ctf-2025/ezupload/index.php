<?php
$action = $_GET['action'] ?? '';
if ($action === 'create') {
  $filename = basename($_GET['filename'] ?? 'phpinfo.php');
  file_put_contents(realpath('.') . DIRECTORY_SEPARATOR . $filename, '<?php phpinfo(); ?>');
  echo "File created.";
} elseif ($action === 'upload') {
  if (isset($_FILES['file']) && $_FILES['file']['error'] === UPLOAD_ERR_OK) {
    $uploadFile = realpath('.') . DIRECTORY_SEPARATOR . basename($_FILES['file']['name']);
    $extension = pathinfo($uploadFile, PATHINFO_EXTENSION);
    if ($extension === 'txt') {
      if (move_uploaded_file($_FILES['file']['tmp_name'], $uploadFile)) {
        echo "File uploaded successfully.";
      }
    }
  }
} else {
  highlight_file(__FILE__);
}
