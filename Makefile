compile:
	brew install mingw-w64
	rustup target add x86_64-pc-windows-gnu

	cargo build --release --target x86_64-pc-windows-gnu

	cd ./target/x86_64-pc-windows-gnu/release && zip bo.zip bo.exe
	cd ./target/x86_64-pc-windows-gnu/release && zip -e bo_encrypted.zip bo.exe
