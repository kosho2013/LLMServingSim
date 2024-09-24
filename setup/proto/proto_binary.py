import setup_pb2
import argparse
from google.protobuf import text_format

# user pass in
parser = argparse.ArgumentParser()
parser.add_argument('--name', type=str, required=True)
args = parser.parse_args()
name = args.name

# read setup.txt
system = setup_pb2.System()
with open('./'+name+'/'+'system.txt', "r") as file:
    text = file.read()
    text_format.Parse(text, system)

# write to binary
with open('./'+name+'/'+'system.bin', "wb") as file:
    file.write(system.SerializeToString())
