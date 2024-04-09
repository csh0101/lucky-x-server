import sys
import os
import shutil
from pathlib import Path

def copy_files(n, target_dir, source_dir):
    # 确保目标目录存在，如果不存在，则创建
    Path(target_dir).mkdir(parents=True, exist_ok=True)
    
    # 获取源目录中的所有文件（简单过滤，可能需要根据实际情况调整）
    files = [f for f in os.listdir(source_dir) if os.path.isfile(os.path.join(source_dir, f))]
    
    # 复制前n个文件到目标目录，如果n大于源目录中的文件数量，则复制所有文件
    for file in files[:n]:
        shutil.copy(os.path.join(source_dir, file), os.path.join(target_dir, file))
        print(f"Copied {file} to {target_dir}")

if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python script.py n target_dir source_dir")
    else:
        n = int(sys.argv[1])
        target_dir = sys.argv[2]
        source_dir = sys.argv[3]
        copy_files(n, target_dir, source_dir)
