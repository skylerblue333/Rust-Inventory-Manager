use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub sku: String,
    pub name: String,
    pub quantity: i64,
}

#[derive(Clone, Default)]
pub struct Inventory {
    items: Arc<Mutex<HashMap<String, Item>>>,
}

impl Inventory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&self, sku: &str, name: &str, quantity: i64) -> Result<Item, &'static str> {
        let sku = sku.trim();
        let name = name.trim();
        if sku.is_empty() || sku.chars().count() > 64 {
            return Err("sku must contain between 1 and 64 characters");
        }
        if name.is_empty() || name.chars().count() > 160 {
            return Err("name must contain between 1 and 160 characters");
        }
        if quantity < 0 {
            return Err("quantity cannot be negative");
        }

        let item = Item {
            sku: sku.to_string(),
            name: name.to_string(),
            quantity,
        };
        let mut items = self.items.lock().map_err(|_| "inventory lock poisoned")?;
        items.insert(sku.to_string(), item.clone());
        Ok(item)
    }

    pub fn get(&self, sku: &str) -> Result<Option<Item>, &'static str> {
        let sku = sku.trim();
        let items = self.items.lock().map_err(|_| "inventory lock poisoned")?;
        Ok(items.get(sku).cloned())
    }

    pub fn list(&self) -> Result<Vec<Item>, &'static str> {
        let items = self.items.lock().map_err(|_| "inventory lock poisoned")?;
        let mut values: Vec<_> = items.values().cloned().collect();
        values.sort_unstable_by(|a, b| a.sku.cmp(&b.sku));
        Ok(values)
    }

    pub fn adjust(&self, sku: &str, delta: i64) -> Result<Item, &'static str> {
        if delta == 0 {
            return Err("delta cannot be zero");
        }
        let sku = sku.trim();
        let mut items = self.items.lock().map_err(|_| "inventory lock poisoned")?;
        let item = items.get_mut(sku).ok_or("sku not found")?;
        let next = item
            .quantity
            .checked_add(delta)
            .ok_or("quantity overflow")?;
        if next < 0 {
            return Err("adjustment would make quantity negative");
        }
        item.quantity = next;
        Ok(item.clone())
    }

    pub fn total_units(&self) -> Result<i128, &'static str> {
        let items = self.items.lock().map_err(|_| "inventory lock poisoned")?;
        Ok(items.values().map(|item| i128::from(item.quantity)).sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_and_list_are_deterministic() {
        let inventory = Inventory::new();
        inventory.upsert("B-2", "Beta", 2).unwrap();
        inventory.upsert("A-1", "Alpha", 3).unwrap();
        let items = inventory.list().unwrap();
        assert_eq!(items[0].sku, "A-1");
        assert_eq!(items[1].sku, "B-2");
        assert_eq!(inventory.total_units().unwrap(), 5);
    }

    #[test]
    fn adjustments_preserve_non_negative_stock() {
        let inventory = Inventory::new();
        inventory.upsert("A-1", "Alpha", 5).unwrap();
        assert_eq!(inventory.adjust("A-1", -2).unwrap().quantity, 3);
        assert!(inventory.adjust("A-1", -4).is_err());
        assert_eq!(inventory.get("A-1").unwrap().unwrap().quantity, 3);
    }

    #[test]
    fn unicode_limits_count_characters() {
        let inventory = Inventory::new();
        let sku = "🚀".repeat(17);
        let item = inventory.upsert(&sku, "全球库存", 1).unwrap();
        assert_eq!(item.sku.chars().count(), 17);
    }

    #[test]
    fn aggregate_quantity_uses_wider_counter() {
        let inventory = Inventory::new();
        inventory.upsert("A", "Alpha", i64::MAX).unwrap();
        inventory.upsert("B", "Beta", i64::MAX).unwrap();
        assert_eq!(inventory.total_units().unwrap(), i128::from(i64::MAX) * 2);
    }

    #[test]
    fn rejects_invalid_items_and_missing_skus() {
        let inventory = Inventory::new();
        assert!(inventory.upsert("", "Alpha", 1).is_err());
        assert!(inventory.upsert("A-1", "", 1).is_err());
        assert!(inventory.upsert("A-1", "Alpha", -1).is_err());
        assert!(inventory.adjust("missing", 1).is_err());
    }
}
