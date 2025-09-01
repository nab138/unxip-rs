# unxip_rs

A library for extracting .xip files, written in Rust.

Note that it uses the `cpio` command which must be installed on your system to run this crate.

## Usage

```rs
fn main() {
    let file = File::open("./Xcode_16.3.xip").unwrap();
    let mut reader = BufReader::new(file);
    let res = unxip(&mut reader, &PathBuf::from("./output"));
    if let Err(e) = res {
        eprintln!("{}", e);
    }
    println!("Done");
}
```

## Credits

- Extraction logic borrowed from [extract_xcode.py](https://github.com/bitcoin-core/apple-sdk-tools/blob/master/extract_xcode.py)
- XAR parsing using [apple-xar](https://crates.io/crates/apple-xar)
