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
        // construct a vector of cells (with ids)
        let data = (0..width * height)
            .map(|x| Cell::new(x))
            .collect::<Vec<Cell>>();

        // split this vector to form a 2D Vec
        //let data: Vec<Vec<Cell>> = a.split_inclusive(|x| (x.id + 1) % width == 0).map(|x| x.to_vec()).collect();

        Self {
            width,
            height,
            data,
        }
    }

    fn manipulate_cell(&mut self, cell_id: usize, drawing: &Drawing) {
        //let (x, y) = self.coord_from_id(cell_id);
        self.data[cell_id].manipulate(drawing);
    }

    fn coord_from_id(&self, cell_id: usize) -> (usize, usize) {
        (cell_id % self.width, (cell_id / self.width) % self.height)
    }

    fn id_from_coord(&self, x: usize, y: usize) -> usize {
        x + self.width * y
    }

    fn get_goal(&self) -> &Cell {
        self.data
            .iter()
            .find(|x| x.cell_type == CellType::Goal)
            .expect("A goal has to be specified")
    }

    //fn set_visisted(&mut self, x: usize, y: usize) {
    //self.data
    //}

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
                //println!("({}, {}): {}", x_n, y_n, id_n);
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
    prev: Box<Option<Cell>>,
}

impl Cell {
    fn new(id: usize) -> Self {
        Self {
            id,
            cell_type: CellType::Default,
            visited: false,
            distance: std::f64::MAX,
            prev: Box::new(None),
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

//struct Agent {
//x: usize,
//y: usize,
//grid: &Grid,
//q: Grid,
//}

//impl Agent {
//fn new(x: usize, y: usize, grid: &Grid) -> Self {
//Self {
//x,
//y,
//grid: Arc::new(grid.clone()),
//q: grid.clone(),
//}
//}

//fn calculate_path(&mut self) -> Vec<Cell> {
////let goal = self.grid.get_goal();
//// dijkstra
//self.dijkstra(&self.grid.data[self.grid.id_from_coord(self.x, self.y)])
//}

fn dijkstra(grid: &mut Grid, start: usize) -> Vec<Cell> {
    let goal = grid.get_goal().clone();

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
    println!("calculating path...");
    let mut cur = prevs.get(&goal.id);
    while let Some(el) = cur {
        //println!("ID: {}", el.id);
        path.push(el.clone());
        cur = prevs.get(&el.id);
    }
    path.reverse();
    path
}

struct Model {
    link: ComponentLink<Self>,
    grid: Grid,
    //agent: Agent,
    drawing: Drawing,
    is_drawing: bool,
}

#[derive(PartialEq)]
enum Drawing {
    Default,
    Obstacle,
    Goal,
}

enum Msg {
    DrawingStart,
    DrawingStop,
    DrawDefault,
    DrawObstacle,
    DrawGoal,
    Start,
    ToggleCell(usize),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let grid = Grid::new(20, 20);
        Self {
            link,
            grid,
            drawing: Drawing::Obstacle,
            is_drawing: false,
            //agent: Agent::new(0, 0, Box<grid>),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DrawingStart => self.is_drawing = true,
            Msg::DrawingStop => self.is_drawing = false,
            Msg::ToggleCell(x) => {
                if self.is_drawing {
                    self.grid.manipulate_cell(x, &self.drawing);
                    for el in self.grid.data.iter_mut() {
                        if el.cell_type == CellType::Path {
                            el.cell_type = CellType::Default;
                        }
                        el.visited = false;
                    }
                    for el in dijkstra(&mut self.grid, 0) {
                        self.grid.data[el.id].cell_type = CellType::Path;
                    }
                }
            }
            Msg::DrawDefault => self.drawing = Drawing::Default,
            Msg::DrawObstacle => self.drawing = Drawing::Obstacle,
            Msg::DrawGoal => self.drawing = Drawing::Goal,
            Msg::Start => {
                for el in self.grid.data.iter_mut() {
                    if el.cell_type == CellType::Path {
                        el.cell_type = CellType::Default;
                    }
                    el.visited = false;
                }
                for el in dijkstra(&mut self.grid, 0) {
                    self.grid.data[el.id].cell_type = CellType::Path;
                }
            }
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
            <div>
                //<input
                <button onclick=self.link.callback(|_| Msg::DrawDefault)>{ "Default" }</button>
                <button onclick=self.link.callback(|_| Msg::DrawObstacle)>{ "Obstacle" }</button>
                <button onclick=self.link.callback(|_| Msg::DrawGoal)>{ "Set Goal" }</button>
                <button onclick=self.link.callback(|_| Msg::Start)>{ "Start" }</button>
                <div class="grid-wrapper">
                    <div class="grid" onmousedown=self.link.callback(|_| Msg::DrawingStart), onmouseup=self.link.callback(|_| Msg::DrawingStop)>
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
                                            }>
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
        //res.iter().map(|x| println!("{:?}\n", grid.coord_from_id(x.id)));
    }
}
