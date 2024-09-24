protoc --python_out=. setup/proto/setup.proto
python setup/proto/proto_binary.py --name=$1
cargo run -r $1 $2 $3 $4 $5