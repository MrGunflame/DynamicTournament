#[derive(Copy, Clone, Debug)]
pub struct Path<'a> {
    path: &'a str,
}

impl<'a> Path<'a> {
    #[inline]
    pub fn new(path: &'a str) -> Self {
        Self { path }
    }

    pub fn take(&mut self) -> Option<&str> {
        if self.path.starts_with('/') {
            self.path = &self.path[1..];
        }

        match self.path.find('/') {
            Some(index) => {
                let path = &self.path[0..index];
                self.path = &self.path[index..];

                Some(path)
            }
            None => {
                if self.path.is_empty() {
                    None
                } else {
                    let path = self.path;
                    self.path = &self.path[0..0];
                    Some(path)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Path;

    #[test]
    fn test_path_take() {
        let mut path = Path::new("");
        assert_eq!(path.take(), None);

        let mut path = Path::new("/");
        assert_eq!(path.take(), None);

        let mut path = Path::new("/1");
        assert_eq!(path.take(), Some("1"));
        assert_eq!(path.take(), None);

        let mut path = Path::new("/1/");
        assert_eq!(path.take(), Some("1"));
        assert_eq!(path.take(), None);

        let mut path = Path::new("/1/2");
        assert_eq!(path.take(), Some("1"));
        assert_eq!(path.take(), Some("2"));
        assert_eq!(path.take(), None);
    }
}
