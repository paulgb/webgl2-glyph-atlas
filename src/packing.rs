// Based on: https://github.com/mackstann/binpack

#[derive(PartialEq, Clone, Copy)]
pub struct RectSize {
    pub height: u32,
    pub width: u32,
}

impl RectSize {
    pub fn fits_in(&self, other: RectSize) -> bool {
        other.width >= self.width && other.height >= self.height
    }

    pub fn area(&self) -> u32 {
        self.height * self.width
    }
}

struct Rect {
    x: u32,
    y: u32,
    size: RectSize,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect {
            x,
            y,
            size: RectSize { height, width },
        }
    }
}

pub struct PackingNode {
    children: Option<(Box<PackingNode>, Box<PackingNode>)>,
    rect: Rect,
    filled: bool,
}

impl PackingNode {
    pub fn new(width: u32, height: u32) -> PackingNode {
        PackingNode {
            children: None,
            rect: Rect::new(0, 0, width, height),
            filled: false,
        }
    }

    fn new_from_rect(rect: Rect) -> PackingNode {
        PackingNode {
            children: None,
            rect,
            filled: false,
        }
    }
}

impl PackingNode {
    pub fn insert_rect(&mut self, rect: RectSize) -> Option<(u32, u32)> {
        if !rect.fits_in(self.rect.size) {
            // If it won't fit, it doesn't matter whether this node is
            // occupied or not, it won't fit.
            return None;
        }

        if let Some((left, right)) = &mut self.children {
            // If a node has children, it is occupied, so we check its children instead.
            let result = left.insert_rect(rect);
            if result.is_some() {
                return result;
            }
            return right.insert_rect(rect);
        }

        if self.filled {
            // A node without children but with filled=true is also occupied, it
            // just happens to be occupied with an exact fit.
            return None;
        }

        if rect == self.rect.size {
            // If the rectangle fits exactly, we don't need to add children.
            self.filled = true;
            return Some((self.rect.x, self.rect.y));
        }

        // Decide whether to partition the node horizontally or vertically.
        // We partition horizontally
        let width_diff = self.rect.size.width - rect.width;
        let height_diff = self.rect.size.height - rect.height;

        let (left_rect, right_rect) = if width_diff > height_diff {
            // Partition horizontally.
            let left_rect = Rect::new(self.rect.x, self.rect.y, rect.width, self.rect.size.height);
            let right_rect = Rect::new(
                self.rect.x + rect.width,
                self.rect.y,
                self.rect.size.width - rect.width,
                self.rect.size.height,
            );

            (left_rect, right_rect)
        } else {
            // Partition vertically.
            let left_rect = Rect::new(self.rect.x, self.rect.y, self.rect.size.width, rect.height);
            let right_rect = Rect::new(
                self.rect.x,
                self.rect.y + rect.height,
                self.rect.size.width,
                self.rect.size.height - rect.height,
            );

            (left_rect, right_rect)
        };

        let mut left_node = PackingNode::new_from_rect(left_rect);
        let result = left_node.insert_rect(rect);

        self.children = Some((
            Box::new(left_node),
            Box::new(PackingNode::new_from_rect(right_rect)),
        ));

        result
    }
}
