mod sketch {
    use crate::sketch;

    #[test]
    fn tree17_4() {
        let sketch = sketch::Tree::new(17, 4);
        assert_eq!(
            sketch.levels(),
            &[
                sketch::Level { index: 0, blocks_count: 1, items_count: 4 },
                sketch::Level { index: 1, blocks_count: 4, items_count: 13 },
            ]
        );
    }

    #[test]
    fn tree17_3() {
        let sketch = sketch::Tree::new(17, 3);
        assert_eq!(
            sketch.levels(),
            &[
                sketch::Level { index: 0, blocks_count: 1, items_count: 3 },
                sketch::Level { index: 1, blocks_count: 3, items_count: 9 },
                sketch::Level { index: 2, blocks_count: 2, items_count: 5 },
            ]
        );
    }
}
