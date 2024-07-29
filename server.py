import os
from flask import Flask, request, render_template_string

app = Flask(__name__)

UPLOAD_FOLDER = './uploads'
if not os.path.exists(UPLOAD_FOLDER):
    os.makedirs(UPLOAD_FOLDER)


@app.route('/')
def upload_form():
    return render_template_string('''
        <!doctype html>
        <title>Upload File</title>
        <h1>Upload File</h1>
        <form method=post enctype=multipart/form-data>
          <input type=file name=file>
          <input type=submit value=Upload>
        </form>
    ''')


@app.route('/', methods=['POST'])
def upload_file():
    if 'file' not in request.files:
        return 'No file part'
    file = request.files['file']
    if file.filename == '':
        return 'No selected file'

    file_path = os.path.join(UPLOAD_FOLDER, file.filename)

    with open(file_path, 'wb') as f:
        while True:
            chunk = file.stream.read(8192)
            if len(chunk) == 0:
                break
            f.write(chunk)

    return 'File successfully uploaded'


if __name__ == "__main__":
    app.run(host='0.0.0.0', port=5555, ssl_context=('cert.pem', 'key.pem'))
