use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

//NOTE this modal was made with much more of a can I rather then a should I mentality.
//And the answer is Yes, yes I can. Should I have ???
//NOTE I am totally not handling error conditions properly
pub struct FileBacked<T: for<'a> Deserialize<'a> + Serialize> {
    inner: T,
    file: PathBuf,
}

impl<T> AsMut<T> for FileBacked<T>
where
    T: for<'a> Deserialize<'a> + Serialize,
{
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> AsRef<T> for FileBacked<T>
where
    T: for<'a> Deserialize<'a> + Serialize,
{
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> FileBacked<T>
where
    T: for<'a> Deserialize<'a> + Serialize,
{
    pub fn new(file: &Path) -> Self {
        let file = file.to_owned();
        Self {
            inner: from_str(&read_to_string(&file).unwrap()).unwrap(),
            file,
        }
    }
}

//NOTE This drop implementation can panic
//and panicking inside of drop is ... not great
//This is fine for my toy project but I would
//hesitate to do this in a production env.
impl<T> Drop for FileBacked<T>
where
    T: for<'a> Deserialize<'a> + Serialize,
{
    fn drop(&mut self) {
        std::fs::write(
            &self.file,
            serde_json::to_string_pretty(&self.inner).unwrap(),
        )
        .unwrap();
    }
}
