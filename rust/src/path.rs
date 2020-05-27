use super::tree::TreeFold;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathJoin<T> {
    Left(T),
    Right(T),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path<T: Clone> {
    pub leaf: T,
    pub join: Vec<PathJoin<T>>,
}

impl<T: Clone> Path<T> {
    pub fn new(leaf: T, join: Vec<PathJoin<T>>) -> Self {
        Self { leaf, join }
    }

    pub fn len(&self) -> usize {
        1 + self.join.len()
    }

    pub fn join_left(&mut self, result: T) {
        self.join.push(PathJoin::Left(result))
    }

    pub fn join_right(&mut self, result: T) {
        self.join.push(PathJoin::Right(result))
    }

    pub fn fold<F>(self, mut f: F) -> T
    where
        F: FnMut(T, T) -> T,
    {
        let mut result = self.leaf;
        for part in self.join {
            result = match part {
                PathJoin::Left(l) => f(l, result),
                PathJoin::Right(r) => f(result, r),
            }
        }
        result
    }
}

pub struct PathTracker<T: TreeFold> {
    base: T,
    input_index: usize,
    stack_index: usize,
    track_input_index: Option<usize>,
    track_stack_index: Option<usize>,
    path: Option<Path<T::Target>>,
    fill: bool,
}

impl<T: TreeFold> PathTracker<T> {
    pub fn new(base: T, track_input_index: Option<usize>) -> Self {
        Self {
            base,
            input_index: 0,
            stack_index: 0,
            track_input_index,
            track_stack_index: None,
            path: None,
            fill: false,
        }
    }

    pub fn path_result(&self) -> Option<Path<T::Target>> {
        self.path.clone()
    }

    pub fn track_index(&mut self, index: usize) {
        // FIXME raise error if index >= input_index
        self.track_input_index.replace(index);
        self.track_stack_index.take();
        self.path.take();
    }

    pub fn track_next(&mut self) {
        self.track_index(self.input_index)
    }
}

impl<T: TreeFold> TreeFold for PathTracker<T> {
    type Leaf = T::Leaf;
    type Target = T::Target;
    type Error = T::Error;

    fn input(&mut self, leaf: &Self::Leaf) -> Result<Self::Target, Self::Error> {
        match self.base.input(leaf) {
            Ok(r) => {
                if !self.fill {
                    if self.track_input_index == Some(self.input_index) {
                        self.path.replace(Path::new(r.clone(), vec![]));
                        self.track_stack_index.replace(self.stack_index + 1);
                    }
                    self.input_index += 1;
                    self.stack_index += 1;
                }
                Ok(r)
            }
            Err(e) => Err(e),
        }
    }

    fn fold(&mut self, a: &Self::Target, b: &Self::Target) -> Result<Self::Target, Self::Error> {
        match self.base.fold(a, b) {
            Ok(r) => {
                if !self.fill {
                    if let Some(idx) = self.track_stack_index {
                        if idx == self.stack_index {
                            self.path.as_mut().unwrap().join_left(a.clone());
                            self.track_stack_index.replace(idx - 1);
                        } else if idx == self.stack_index - 1 {
                            self.path.as_mut().unwrap().join_right(b.clone());
                        }
                    }
                    self.stack_index -= 1;
                }
                Ok(r)
            }
            Err(e) => Err(e),
        }
    }

    fn start_fill(&mut self) {
        self.fill = true;
    }

    fn end_fill(&mut self) {
        self.fill = false;
        self.stack_index += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tree::test::TestFold;
    use crate::tree::TreeFolder;

    #[test]
    fn test_track() {
        let leaves = (0..8).map(|n| n.to_string());
        let tracker = PathTracker::new(TestFold {}, Some(3));
        let (result, tracker) = TreeFolder::fold(tracker, leaves, None).unwrap();
        let path = tracker.path_result().unwrap();
        let expect_result = "[[[0,1],[2,3]],[[4,5],[6,7]]]";
        assert_eq!(result.unwrap(), expect_result);
        assert_eq!(
            path,
            Path::new(
                "3".to_string(),
                vec![
                    PathJoin::Left("2".to_string()),
                    PathJoin::Left("[0,1]".to_string()),
                    PathJoin::Right("[[4,5],[6,7]]".to_string())
                ]
            )
        );
        assert_eq!(path.fold(|l, r| format!("[{},{}]", l, r)), expect_result);
    }

    #[test]
    fn test_track_multiple() {
        let leaves: Vec<String> = (0..128).map(|n| n.to_string()).collect();
        for i in 0..128 {
            let tracker = PathTracker::new(TestFold {}, Some(i));
            let (result, tracker) = TreeFolder::fold(tracker, leaves.clone(), None).unwrap();
            let path = tracker.path_result().unwrap();
            let leaf = path.leaf.to_string();
            let verify = path.fold(|l, r| format!("[{},{}]", l, r));
            assert_eq!(leaf, i.to_string());
            assert_eq!(result.unwrap(), verify);
        }
    }
}
