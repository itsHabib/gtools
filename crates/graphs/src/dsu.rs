pub(crate) struct DisjointSet {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl DisjointSet {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    pub fn find(&mut self, v: usize) -> usize {
        assert!(v < self.parent.len(), "{v} not in bounds");

        if  v >= self.parent.len() {
            return v;
        }

        let p = self.parent[v];
        if p != v {
            self.parent[v] = self.find(p);
        }

        self.parent[v]
    }

    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);

        if ra == rb {
            return false;
        }

        let ra_size = self.size[ra];
        let rb_size = self.size[rb];

        if ra_size >= rb_size {
            self.parent[rb] = ra;
            self.size[ra] += self.size[rb];
        } else {
            self.parent[ra] = rb;
            self.size[rb] += self.size[ra];
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut ds = DisjointSet::new(5);
        assert_ne!(ds.find(0), ds.find(1));
        assert!(ds.union(0, 1));
        assert_eq!(ds.find(0), ds.find(1));
        assert!(!ds.union(0, 1));
    }

    #[test]
    fn test_transitive() {
        let mut ds = DisjointSet::new(5);
        ds.union(0, 1);
        ds.union(1, 2);
        let root = ds.find(0);
        assert_eq!(ds.find(1), root);
        assert_eq!(ds.find(2), root);
        assert_ne!(ds.find(3), root);
    }

    #[test]
    fn test_multiple() {
        let mut ds = DisjointSet::new(6);
        ds.union(0, 1);
        ds.union(1, 2);
        ds.union(3, 4);
        assert_eq!(ds.find(0), ds.find(2));
        assert_eq!(ds.find(3), ds.find(4));
        assert_ne!(ds.find(0), ds.find(3));
    }

    #[test]
    #[should_panic(expected = "not in bounds")]
    fn test_out_of_bounds() {
        let mut ds = DisjointSet::new(5);
        ds.find(10);
    }
}