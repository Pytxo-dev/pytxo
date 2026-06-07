use crate::fetch::CatalogModel;

pub fn search_models(models: &[CatalogModel], query: &str) -> Vec<CatalogModel> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return models.to_vec();
    }
    let mut scored: Vec<(i32, CatalogModel)> = models
        .iter()
        .filter_map(|m| {
            let id = m.id.to_ascii_lowercase();
            let name = m.name.to_ascii_lowercase();
            let mut score = 0i32;
            if id == q || name == q {
                score += 100;
            } else if id.contains(&q) || name.contains(&q) {
                score += 50;
            } else if q
                .split_whitespace()
                .all(|w| id.contains(w) || name.contains(w))
            {
                score += 25;
            } else {
                return None;
            }
            Some((score, m.clone()))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
    scored.into_iter().map(|(_, m)| m).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_matches_substring() {
        let models = vec![CatalogModel {
            id: "deepseek-chat".into(),
            name: "DeepSeek Chat".into(),
            provider: "deepseek".into(),
        }];
        let out = search_models(&models, "deep");
        assert_eq!(out.len(), 1);
    }
}
