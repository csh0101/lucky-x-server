import argparse
import hashlib
import os
import os.path
import zipfile
import uuid
from concurrent.futures import ThreadPoolExecutor, as_completed
import re


def getdir(sm):
    num = re.sub("\D", "", sm)
    mstr = re.sub("\d", "", sm)

    m2 = hashlib.md5()
    m2.update(num.encode("utf-8"))
    num_md5 = m2.hexdigest()

    m3 = hashlib.md5()
    m3.update(mstr.encode("utf-8"))
    str_md5 = m3.hexdigest()

    num1 = re.sub("\D", "", num_md5)
    num2 = re.sub("\D", "", str_md5)
    res = num1 + num2
    res = res[-10:]
    res = int(res)
    n = 0
    dir = ""
    while n < 3:
        n += 1
        dir += "/" + str(res % 1000)
        res = res // 1000
    return dir


def read_file(pathfile):
    """Read file content."""
    with open(pathfile, "rb") as f:
        content = f.read()
    return content


def makeZip(output_filename, folder):
    file_contents = {}
    with ThreadPoolExecutor(max_workers=200) as executor:
        future_to_file = {
            executor.submit(read_file, os.path.join(parent, onefile)): (parent, onefile)
            for parent, _, filenames in os.walk(folder)
            for onefile in filenames
        }

        for future in as_completed(future_to_file):
            parent, onefile = future_to_file[future]
            try:
                data = future.result()
                pathfile = os.path.join(parent, onefile)
                pre_len = len(os.path.dirname(folder))
                arcname = pathfile[pre_len:].strip(
                    os.path.sep
                )  # Get relative path of file
                file_contents[arcname] = data
            except Exception as exc:
                print(f"{onefile} generated an exception: {exc}")

    try:
        with zipfile.ZipFile(output_filename, "w") as zipf:
            for arcname, data in file_contents.items():
                zipf.writestr(arcname, data)
        print(f"ZIP file {output_filename} has been created.")
        return True
    except Exception as e:
        print(str(e))
        return False


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Create a ZIP file of a directory with a random name."
    )
    parser.add_argument("path", type=str, help="Path of the directory to zip.")
    args = parser.parse_args()

    # Generate a random filename
    random_filename = str(uuid.uuid4()) + ".zip"
    makeZip(random_filename, args.path)
