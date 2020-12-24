#![recursion_limit = "4096"]
#![feature(split_inclusive, vec_remove_item, total_cmp)]

use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Clone)]
struct Grid {
    width: usize,
    height: usize,
    data: Vec<Cell>,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        let data = (0..width * height)
            .map(|x| Cell::new(x, width))
            .collect::<Vec<Cell>>();

        Self {
            width,
            height,
            data,
        }
    }

    fn manipulate_cell(&mut self, cell_id: usize, drawing: &Drawing) {
        self.data[cell_id].manipulate(drawing);
    }

    fn coord_from_id(&self, cell_id: usize) -> (usize, usize) {
        (cell_id % self.width, (cell_id / self.width) % self.height)
    }

    fn id_from_coord(&self, x: usize, y: usize) -> usize {
        x + self.width * y
    }

    fn get_goal(&self) -> Option<&Cell> {
        self.data
            .iter()
            .find(|x| x.cell_type == CellType::Goal)
    }

    fn smallest_distance(&self) -> &Cell {
        self.data
            .iter()
            .min_by(|x, y| x.distance.total_cmp(&y.distance))
            .unwrap()
    }

    fn get_neighbors(&self, cell: &Cell) -> Vec<&Cell> {
        let mut perm: Vec<&Cell> = vec![];
        let (x, y) = self.coord_from_id(cell.id);
        for i in -1..2 {
            for j in -1..2 {
                let x_n = x as isize + i;
                let y_n = y as isize + j;
                if x_n < 0 || x_n >= self.width as isize || y_n < 0 || y_n >= self.height as isize {
                    continue;
                }
                let id_n = self.id_from_coord(x_n as usize, y_n as usize);
                let cell = &self.data[id_n];
                if cell.visited || cell.cell_type == CellType::Obstacle {
                    continue;
                }
                perm.push(cell);
            }
        }
        perm
    }

    fn dist(&self, u: &Cell, v: &Cell) -> f64 {
        let (u_x, u_y) = self.coord_from_id(u.id);
        let (v_x, v_y) = self.coord_from_id(v.id);

        (((u_x as isize - v_x as isize).pow(2) + (u_y as isize - v_y as isize).pow(2)) as f64)
            .sqrt()
    }

    fn rows(&self) -> Vec<Vec<Cell>> {
        let data: Vec<Vec<Cell>> = self
            .data
            .split_inclusive(|x| (x.id + 1) % self.width == 0)
            .map(|x| x.to_vec())
            .collect();
        data
    }
}

#[derive(Clone, PartialEq)]
struct Cell {
    id: usize,
    cell_type: CellType,
    visited: bool,
    distance: f64,
    side_length: f64,
}

impl Cell {
    fn new(id: usize, total_width: usize) -> Self {
        Self {
            id,
            cell_type: CellType::Default,
            visited: false,
            distance: std::f64::MAX,
            side_length: (100 as f64 - total_width as f64) / total_width as f64,
        }
    }

    fn class_str(&self) -> &str {
        match self.cell_type {
            CellType::Default => "cell default",
            CellType::Obstacle => "cell obstacle",
            CellType::Goal => "cell goal",
            CellType::Path => "cell path",
        }
    }

    fn style_str(&self) -> String {
        format!("width:{}vw;height:{}vw;", self.side_length, self.side_length)
    }

    fn id(&self) -> usize {
        self.id
    }

    fn manipulate(&mut self, drawing: &Drawing) {
        self.cell_type = match drawing {
            Drawing::Default => CellType::Default,
            Drawing::Obstacle => CellType::Obstacle,
            Drawing::Goal => CellType::Goal,
        };
    }
}

#[derive(Copy, Clone, PartialEq)]
enum CellType {
    Default,
    Obstacle,
    Goal,
    Path,
}

fn dijkstra(grid: &mut Grid, start: usize) -> Vec<Cell> {
    let goal = grid.get_goal().unwrap().clone();

    let mut q = grid.clone();
    // initialize
    for x in q.data.iter_mut() {
        x.distance = std::f64::MAX;
    }
    //self.q.data.iter_mut().map(|x| x.distance = std::f64::MAX);
    q.data[start].distance = 0.0;

    let mut path: Vec<Cell> = vec![];
    let mut prevs = HashMap::new();

    while !q.data.is_empty() {
        // remove u from q
        let cell = q.smallest_distance().clone();
        q.data.remove_item(&cell);
        grid.data[cell.id.clone()].visited = true;
        for neighbor in grid.get_neighbors(&cell) {
            if let Some(el) = q.data.iter_mut().find(|x| x.id == neighbor.id) {
                // update distance
                let alt = cell.distance + (grid.dist(&cell, el));
                if alt < el.distance {
                    el.distance = alt;
                    prevs.insert(el.id.clone(), cell.clone());
                }
                if el.cell_type == CellType::Goal {
                    break;
                }
            }
        }
    }
    let mut cur = prevs.get(&goal.id);
    while let Some(el) = cur {
        path.push(el.clone());
        cur = prevs.get(&el.id);
    }
    path.reverse();
    path
}

struct Model {
    link: ComponentLink<Self>,
    grid: Grid,
    drawing: Drawing,
    is_drawing: bool,
}

impl Model {
    fn button_class_str(&self, msg: &Msg) -> &str {
        if msg == &Msg::DrawDefault && self.drawing == Drawing::Default ||
           msg == &Msg::DrawObstacle && self.drawing == Drawing::Obstacle ||
           msg == &Msg::DrawGoal && self.drawing == Drawing::Goal {
               "toggled"
        }
        else {
            ""
        }
    }

    fn toggle_cell(&mut self, cell_id: usize) {
        self.grid.manipulate_cell(cell_id, &self.drawing);
        for el in self.grid.data.iter_mut() {
            if el.cell_type == CellType::Path {
                el.cell_type = CellType::Default;
            }
            el.visited = false;
        }
        if self.grid.get_goal().is_some() {
            for el in dijkstra(&mut self.grid, 0) {
                self.grid.data[el.id].cell_type = CellType::Path;
            }
        }
    }
}

#[derive(PartialEq)]
enum Drawing {
    Default,
    Obstacle,
    Goal,
}

#[derive(PartialEq)]
enum Msg {
    DrawingStart,
    DrawingStop,
    DrawDefault,
    DrawObstacle,
    DrawGoal,
    ToggleCell(usize),
    ForceToggle(usize),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let grid = Grid::new(30, 30);
        Self {
            link,
            grid,
            drawing: Drawing::Obstacle,
            is_drawing: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DrawingStart => self.is_drawing = true,
            Msg::DrawingStop => self.is_drawing = false,
            Msg::ToggleCell(x) => {
                if self.is_drawing {
                    self.toggle_cell(x);
                }
            }
            Msg::ForceToggle(x) => self.toggle_cell(x),
            Msg::DrawDefault => self.drawing = Drawing::Default,
            Msg::DrawObstacle => self.drawing = Drawing::Obstacle,
            Msg::DrawGoal => self.drawing = Drawing::Goal,
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="site">
                //<input
                <button class=self.button_class_str(&Msg::DrawDefault) onclick=self.link.callback(|_| Msg::DrawDefault)>{ "Default" }</button>
                <button class=self.button_class_str(&Msg::DrawObstacle) onclick=self.link.callback(|_| Msg::DrawObstacle)>{ "Obstacle" }</button>
                <button class=self.button_class_str(&Msg::DrawGoal) onclick=self.link.callback(|_| Msg::DrawGoal)>{ "Set Goal" }</button>
                <div class="grid-wrapper">
                    <div class="grid" onmousedown=self.link.callback(|_| Msg::DrawingStart) ontouchstart=self.link.callback(|_| Msg::DrawingStart), onmouseup=self.link.callback(|_| Msg::DrawingStop) ontouchend=self.link.callback(|_| Msg::DrawingStop)>
                    {
                        for self.grid.rows().iter()
                            .map(|v| html! {
                                <div class="row">
                                    { for v.iter()
                                        .map(|x| html! {
                                            <div class=x.class_str() id=x.id() onmousemove=
                                            {
                                                let cpy = x.clone();
                                                self.link.callback(move |_| Msg::ToggleCell(cpy.id().clone()))
                                            } ontouchmove=
                                            {
                                                let cpy = x.clone();
                                                self.link.callback(move |_| Msg::ToggleCell(cpy.id().clone()))
                                            } onclick=
                                            {
                                                let cpy = x.clone();
                                                self.link.callback(move |_| Msg::ForceToggle(cpy.id().clone()))
                                            }
                                            style=x.style_str()>
                                            </div> })
                                    }
                                </div>
                                }
                            )
                    }
                    </div>
                </div>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dijkstra_test() {
        let mut grid = Grid::new(10, 10);
        grid.data[98].cell_type = CellType::Goal;
        let res = dijkstra(&mut grid, 0);
        for x in res {
            println!("{:?} - {}", grid.coord_from_id(x.id), x.id);
        }
    }
}
