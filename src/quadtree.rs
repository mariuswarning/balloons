use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::iter::FromIterator;
use web_sys::console;
use crate::ball::{Ball, BallPair, BallPairIds, is_ball_in_cell, is_ball_in_cell_diag};
use itertools::{all, any, Itertools};
use crate::geometry::{Cells, Rect, RenderingRect};
use crate::logic;

#[derive(Debug)]
pub struct QuadTreeNode {
    rect: Rect,
    id: usize,
    depth: u8,
    balls: HashMap<usize, Ball>,
    top_left: ChildNode,
    top_right: ChildNode,
    bottom_left: ChildNode,
    bottom_right: ChildNode,
}

pub type ChildNode = Option<Box<QuadTreeNode>>;

impl Clone for QuadTreeNode {
    fn clone(&self) -> Self {
        Self {
            rect: self.rect.clone(),
            id: self.id,
            depth: self.depth,
            balls: self.balls.clone(),
            top_left: self.top_left.clone(),
            top_right: self.top_right.clone(),
            bottom_left: self.bottom_left.clone(),
            bottom_right: self.bottom_right.clone(),
        }
    }
}


impl QuadTreeNode {
    pub fn new(rect: Rect, id: usize, depth: u8) -> ChildNode {
        //console::log_1(&format!("QTN {} has {}", id, size).into());
        let balls: HashMap<usize, Ball> = HashMap::new();

        Option::from(Box::new(Self {
            rect,
            id,
            depth,
            balls,
            top_left: None,
            top_right: None,
            bottom_left: None,
            bottom_right: None,
        }))
    }

    pub fn new_top_left(&self) -> ChildNode {
        QuadTreeNode::new(self.rect.top_left(), self.id * 10 + 1, self.depth + 1)
    }

    pub fn new_top_right(&self) -> ChildNode {
        QuadTreeNode::new(self.rect.top_right(), self.id * 10 + 2, self.depth + 1)
    }

    pub fn new_bottom_left(&self) -> ChildNode {
        QuadTreeNode::new(self.rect.bottom_left(), self.id * 10 + 3, self.depth + 1)
    }

    pub fn new_bottom_right(&self) -> ChildNode {
        QuadTreeNode::new(self.rect.bottom_right(), self.id * 10 + 4, self.depth + 1)
    }

    pub fn contains(&self, id: usize) -> bool {
        self.balls.contains_key(&id)
    }

    fn child_node_contains(child: &ChildNode, id: usize) -> bool {
        if child.is_some() {
            return child.as_ref().unwrap().contains(id);
        }
        false
    }

    pub fn contains_in_children(&self, id: usize) -> bool {
        if QuadTreeNode::child_node_contains(&self.top_left, id)
            || QuadTreeNode::child_node_contains(&self.top_right, id)
            || QuadTreeNode::child_node_contains(&self.bottom_left, id)
            || QuadTreeNode::child_node_contains(&self.bottom_right, id)
        {
            return true;
        }
        false
    }

    pub fn insert_ball(&mut self, ball: &Ball) {
        let node_cell = self.rect.borrow();
        let new_cells = node_cell.split_to_four_cells();
        let mut child_node: ChildNode;
        let ball_box = ball.bounding_rect_current();
        //console::log_1(&format!("inserting ball {} into {} ball box{:#?} into {:#?}", ball.id, self.id, ball_box, self.rect).into());

        // if !ball_box.fits(self.rect) {
        //     console::log_1(&format!("dosn't fit, returning inserting ball {} {} {:#?} {:#?}", ball.id, self.id, ball_box, self.rect).into());
        //     return;
        // }
        self.balls.insert(ball.id, *ball);

        // which cell does the ball belong to?
        match node_cell.where_is_point_relative_to_center(ball.pos) {
            Cells::TopLeft => {
                //console::log_1(&format!("top-left -> inserting ball {} {} {:#?}", ball.id, self.id, new_cells.0,).into());

                if !ball_box.fits(new_cells.0) {
                    //console::log_1(&format!(" doesn't fit into top-left -> inserting ball into parent {} {} {:#?} {:#?} {:#?}", ball.id, self.id, self.rect, ball.pos, new_cells.0, ).into());
                    return;
                }

                if self.top_left.is_none() {
                    self.top_left = self.new_top_left();
                }
                self.top_left.as_mut().unwrap().insert_ball(ball);
            }
            Cells::TopRight => {
                if !ball_box.fits(new_cells.1) {
                    return;
                }

                if self.top_right.is_none() {
                    self.top_right = self.new_top_right();
                }
                self.top_right.as_mut().unwrap().insert_ball(ball);
            }
            Cells::BottomLeft => {
                if !ball_box.fits(new_cells.2) {
                    return;
                }

                if self.bottom_left.is_none() {
                    self.bottom_left = self.new_bottom_left();
                }
                self.bottom_left.as_mut().unwrap().insert_ball(ball);
            }
            Cells::BottomRight => {
                if !ball_box.fits(new_cells.3) {
                    return;
                }

                if self.bottom_right.is_none() {
                    self.bottom_right = self.new_bottom_right();
                }
                self.bottom_right.as_mut().unwrap().insert_ball(ball);
            }
        }
    }

    pub fn search(&self, id: usize) -> Vec<BallPairIds> {
        //let mut rez: Vec<BallPairIds> = vec![];
        let mut pairs: Vec<BallPairIds> = vec![];

        if !self.contains(id) {
            return vec![];
        }
        if !self.contains_in_children(id) {
            let combos = Vec::from_iter(self.balls.keys().cloned().combinations(2));
            for c in combos {
                if c[0] == id || c[1] == id {
                    pairs.push(BallPairIds { first: c[0], second: c[1] })
                }
            }
            return pairs;
        }

        if self.top_left.is_some() {
            let node = self.top_left.as_ref().unwrap();
            pairs.extend(node.search(id));
        }
        if self.top_right.is_some() {
            let node = self.top_right.as_ref().unwrap();
            pairs.extend(node.search(id));
        }
        if self.bottom_left.is_some() {
            let node = self.bottom_left.as_ref().unwrap();
            pairs.extend(node.search(id));
        }
        if self.bottom_right.is_some() {
            let node = self.bottom_right.as_ref().unwrap();
            pairs.extend(node.search(id));
        }

        return pairs;
    }

    pub fn get_rectangles(&self) -> Vec<RenderingRect> {
        let x = self.rect.x;
        let y = self.rect.y;
        let w = self.rect.w;
        let h = self.rect.h;
        let mut many = false;
        let mut rects = vec![];

        if w < 2.0 {
            return vec![];
        }

        if self.top_left.is_some() {
            let tl = self.top_left.as_ref().unwrap();
            let additional = tl.get_rectangles();
            rects.extend(additional);
        }

        if self.top_right.is_some() {
            let tl = self.top_right.as_ref().unwrap();
            let additional = tl.get_rectangles();
            rects.extend(additional);
        }

        if self.bottom_left.is_some() {
            let tl = self.bottom_left.as_ref().unwrap();
            let additional = tl.get_rectangles();
            rects.extend(additional);
        }

        if self.bottom_right.is_some() {
            let tl = self.bottom_right.as_ref().unwrap();
            let additional = tl.get_rectangles();
            rects.extend(additional);
        }
        many = all(vec!(self.bottom_left.as_ref(), self.bottom_right.as_ref(), self.top_left.as_ref(), self.top_right.as_ref()),
                   |item| item.is_none()) && self.balls.len() > 1;
        let r: RenderingRect = RenderingRect { rect: Rect { x, y, w, h }, many };
        rects.push(r);

        return rects;
    }


    pub fn info_collisions(&self) -> Vec<String> {
        let mut lines = vec![];

        // if (self.is_collision_candidate || self.children_are_collision_candidates) {
        //     let keys: Vec<String> = self.balls.keys().map(|key| key.to_string()).collect();
        //     let keyo = keys.join(",");
        //     lines.push(format!(" {} is:{} children:{} balls:{:#?}", self.id, self.is_collision_candidate, self.children_are_collision_candidates, keyo));
        // } else {
        //     return lines;
        // }
        //

        if self.top_left.is_some() {
            let node = self.top_left.as_ref().unwrap();
            lines.extend(node.info_collisions());
        }

        if self.top_right.is_some() {
            let node = self.top_right.as_ref().unwrap();
            lines.extend(node.info_collisions());
        }


        if self.bottom_left.is_some() {
            let node = self.bottom_left.as_ref().unwrap();
            lines.extend(node.info_collisions());
        }

        if self.bottom_right.is_some() {
            let node = self.bottom_right.as_ref().unwrap();
            lines.extend(node.info_collisions());
        }

        return lines;
    }

    pub fn info_balls(&self) -> Vec<String> {
        let mut lines = vec![];

        for id in self.balls.keys().sorted() {
            let ball = self.balls.get(id).expect("");
            lines.push(format!(
                "{} {} {} {} {}", ball.ball_type, ball.pos.x, ball.pos.y, ball.next_position.x, ball.next_position.y));

        }
        return lines;
    }

    pub fn info_ball_quads(&self) -> Vec<String> {
        let mut lines = vec![];

        for id in self.balls.keys().sorted() {
            let quad_ids = self.search(*id);
            let quad_ids_str: Vec<String> = quad_ids
                .iter()
                .cloned()
                .map(|combo|
                    combo.into_array()
                         .map(|ball_id| ball_id.to_string())
                         .join("|"))
                .collect();
            let line = quad_ids_str.join(",");
            lines.push(format!(" ball id:{} {} is in {}", id, self.balls.get(id).expect("").color.to_string(), line));
        }

        return lines;
    }
}

//
// fn get_child_collisions(node: &ChildNode) -> Vec<BallPair> {
//     if node.is_some() {
//         let cands = node.as_ref().unwrap().find_collision_candidates();
//         if cands.len() > 0 {
//             //console::log_1(&format!("candidates in this node: {:#?}", cands).into());
//             return cands;
//         }
//     }
//     return vec![];
// }




