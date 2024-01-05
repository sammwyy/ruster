pub fn random_element(vec: Vec<String>) -> Option<String> {
    if vec.is_empty() {
        return None;
    }

    let random_index = rand::random::<usize>() % vec.len();
    let element = vec[random_index].clone();
    Some(element)
}
