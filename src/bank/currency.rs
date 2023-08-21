use strum_macros::ToString;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, ToString)]
pub enum Currency {
    USD,
    OSMO,
}
