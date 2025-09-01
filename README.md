# unxip_rs

A library for extracting .xip files, written in Rust.

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
