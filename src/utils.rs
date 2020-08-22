use curl::easy::Easy;

pub type Result<T> = std::result::Result<T, ()>;
pub fn err<T>() -> std::result::Result<T,()> {
    Err(())
}

pub fn ok<T>(t: T) -> Result<T> {
    Ok(t)
}

pub fn get_string(url: &str) -> Result<String> {
    get_vec(url).map_or(
        Err(()),
        |v| String::from_utf8(v).map_or(Err(()), |s| Ok(s)))
}

pub fn get_vec(url: &str) -> Result<Vec<u8>> {
 
    // get the html for GUI
    let mut data = Vec::new();
    let mut handle = Easy::new();
 
    if let Err(_) = handle.url(url) {
        return Err(());
    }

    {    
        let mut transfer = handle.transfer();
        if let Err(_) = transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }) {
            return Err(());   
        }

        if let Err(_) = transfer.perform() {
            return Err(());
        }
    }

    Ok(data)
}