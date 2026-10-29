use crate::shared::{Failure, Result, require};
use serde_json::{Value, json};
/// Optional additive pagination: legacy requests keep their original array response.
pub(crate) fn paginate(value: Value, limit: Option<usize>, cursor: Option<&str>) -> Result<Value> {
    let Some(limit) = limit else {
        require(cursor.is_none(), "游标须配合分页大小使用")?;
        return Ok(value);
    };
    require((1..=200).contains(&limit), "分页大小须在1至200之间")?;
    let rows = value
        .as_array()
        .ok_or(Failure("INVALID_INPUT", "该资源不支持列表分页"))?;
    let mut entries: Vec<(String, Value)> = rows
        .iter()
        .map(|v| {
            let id = v["id"]
                .as_str()
                .or_else(|| v["key"].as_str())
                .ok_or(Failure("INVALID_INPUT", "列表缺少稳定编号"))?;
            Ok((id.to_owned(), v.clone()))
        })
        .collect::<Result<_>>()?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    require(
        entries.windows(2).all(|p| p[0].0 != p[1].0),
        "列表存在重复编号，需先核查",
    )?;
    if let Some(cursor) = cursor {
        require(cursor.len() <= 512, "游标过长")?;
        entries.retain(|(id, _)| id.as_str() > cursor);
    }
    let more = entries.len() > limit;
    entries.truncate(limit);
    let next = if more {
        entries.last().map(|(id, _)| id.clone())
    } else {
        None
    };
    Ok(
        json!({"items":entries.into_iter().map(|(_,v)|v).collect::<Vec<_>>(),"next_cursor":next,"sort":"id_asc","limit":limit}),
    )
}
