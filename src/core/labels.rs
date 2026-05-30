use crate::game::*;

pub(super) fn development_focus_label(focus: &DevelopmentFocus) -> &'static str {
    match focus {
        DevelopmentFocus::Agriculture => "农业",
        DevelopmentFocus::Commerce => "商业",
        DevelopmentFocus::Defense => "城防",
        DevelopmentFocus::Order => "治安",
    }
}

pub(super) fn diplomacy_label(proposal: &DiplomacyProposal) -> &'static str {
    match proposal {
        DiplomacyProposal::ImproveRelations => "改善关系",
        DiplomacyProposal::Truce => "停战",
        DiplomacyProposal::DeclareWar => "宣战",
    }
}

pub(super) fn city_scale_label(scale: &CityScale) -> &'static str {
    match scale {
        CityScale::County => "县城",
        CityScale::Commandery => "郡治",
        CityScale::RegionalCapital => "州郡重镇",
        CityScale::ImperialCapital => "都城",
    }
}

pub(super) fn confidence_label(confidence: &SourceConfidence) -> &'static str {
    match confidence {
        SourceConfidence::High => "高",
        SourceConfidence::Medium => "中",
        SourceConfidence::Low => "低",
    }
}
