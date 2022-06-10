mod runtime;
mod quality;

pub use runtime::*;
pub use quality::*;

pub fn get_symmetric_problems_with_opt() -> Vec<(&'static str, u32)> {
    vec![
        ("hk48", 11461),
        ("gr48", 5046),
        ("eil51", 426),
        ("berlin52", 7542),
        ("st70", 675),
        ("eil76", 538),
        ("pr76", 108159),
        ("rat99", 1211),
        ("kroA100", 21282),
        ("kroB100", 22141),
        ("kroC100", 20749),
        ("kroD100", 21294),
        ("kroE100", 22068),
        ("rd100", 7910),
        ("eil101", 629),
        ("lin105", 14379),
        ("pr107", 44303),
        ("gr120", 6942),
        ("pr124", 59030),
        ("bier127", 118282),
        ("ch130", 6110),
        ("pr136", 96772),
        ("pr144", 58537),
        ("ch150", 6528),
        ("kroA150", 26524),
        ("kroB150", 26130),
        ("pr152", 73682),
        ("u159", 42080),
    ]
}

pub fn get_asymmetric_problems_with_opt() -> Vec<(&'static str, u32)> {
    vec![
        ("ftv33", 1286),
        ("ftv35", 1473),
        ("ftv38", 1530),
        ("ftv44", 1613),
        ("ftv47", 1776),
        ("ftv55", 1608),
        ("ftv64", 1839),
        ("ftv70", 1950),
    ]
}
