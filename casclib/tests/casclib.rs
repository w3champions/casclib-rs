const STROAGE_PATH: &str = r#"C:\Program Files (x86)\Warcraft III\Data"#;
const TEST_FILE: &str = r#"war3.w3mod:scripts\blizzard.j"#;

#[test]
#[ignore = "requires a local Warcraft III install at the hardcoded STROAGE_PATH"]
fn test_all() {
    let storage = casclib::open(STROAGE_PATH).unwrap();
    let count = storage.file_count();
    assert!(count > 0);

    let mut map_file_count = 0;
    for r in storage.files_with_mask("war3.w3mod:maps") {
        let entry = r.expect("file entry");
        let _name = entry.get_name();
        map_file_count += 1;
    }

    assert!(map_file_count > 0);

    let _bj = storage.entry(TEST_FILE).open().unwrap();
}

#[cfg(target_os = "windows")]
#[test]
#[ignore = "requires a local Warcraft III install at the hardcoded unicode path"]
fn test_read_unicode() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use widestring::U16CString;

    let storage = casclib::open(OsString::from_wide(
        &U16CString::from_str(r#"C:\Program Files (x86)\Warcraft III中文\Data"#)
            .unwrap()
            .into_vec(),
    ))
    .unwrap();
    let count = storage.file_count();

    assert!(count > 0);

    let mut map_file_count = 0;
    for r in storage.files_with_mask("war3.w3mod:maps") {
        let entry = r.expect("file entry");
        let _name = entry.get_name();
        map_file_count += 1;
    }

    assert!(map_file_count > 0);

    let _bj = storage.entry(TEST_FILE).open().unwrap();
}
