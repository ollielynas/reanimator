
import os
import tarfile

os.system('cargo dist build')

print("finished building")

def make_tarfile(output_filename, source_dir):
    with tarfile.open(output_filename, "w:gz") as tar:
        tar.add(source_dir, arcname=os.path.basename(source_dir))
        

make_tarfile("target/distrib/reanimator-x86_64-pc-windows-msvc.tar.gz", "target/distrib/reanimator-x86_64-pc-windows-msvc/")


os.system('copy target\\distrib\\reanimator-x86_64-pc-windows-msvc\\reanimator.exe target\\distrib\\reanimator-x86_64-pc-windows-msvc-a.exe')

print("finished build 2")