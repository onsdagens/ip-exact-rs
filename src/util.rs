use roxmltree::Node;

pub fn find_child_by_tag_name<'a>(node: Node<'a, '_>, name: &str) -> Option<Node<'a, 'a>> {
    node.children().find(|child| child.has_tag_name(name))
}

pub fn get_name(node: Node) -> Option<String> {
    Some(
        node.children()
            .find(|child| child.has_tag_name("name"))?
            .text_storage()?
            .to_string(),
    )
}

pub fn find_descendant_by_tag_name<'a>(node: Node<'a, '_>, name: &str) -> Option<Node<'a, 'a>> {
    node.descendants()
        .find(|descendant| descendant.has_tag_name(name))
}

pub fn find_children_by_tag_name<'a>(node: Node<'a, '_>, name: &str) -> Vec<Node<'a, 'a>> {
    node.children()
        .filter(|child| child.has_tag_name(name))
        .collect()
}

pub fn find_descendants_by_tag_name<'a>(node: Node<'a, '_>, name: &str) -> Vec<Node<'a, 'a>> {
    node.descendants()
        .filter(|descendant| descendant.has_tag_name(name))
        .collect()
}
