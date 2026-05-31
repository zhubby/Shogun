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

pub(super) fn facility_kind_label(kind: FacilityKind) -> &'static str {
    match kind {
        FacilityKind::Farmland => "农田",
        FacilityKind::Irrigation => "水利",
        FacilityKind::Market => "市集",
        FacilityKind::TradeDepot => "商站",
        FacilityKind::Workshop => "工坊",
        FacilityKind::Quarry => "采石场",
        FacilityKind::Barracks => "兵营",
        FacilityKind::DrillGround => "校场",
        FacilityKind::Walls => "城墙",
        FacilityKind::Administration => "官署",
        FacilityKind::Granary => "粮仓",
        FacilityKind::RelayStation => "驿站",
    }
}

pub(super) fn confidence_label(confidence: &SourceConfidence) -> &'static str {
    match confidence {
        SourceConfidence::High => "高",
        SourceConfidence::Medium => "中",
        SourceConfidence::Low => "低",
    }
}

pub(super) fn officer_gender_label(gender: &OfficerGender) -> &'static str {
    match gender {
        OfficerGender::Male => "男",
        OfficerGender::Female => "女",
    }
}

pub(super) fn officer_relationship_label(kind: &OfficerRelationshipKind) -> &'static str {
    match kind {
        OfficerRelationshipKind::RulerSubject => "君臣",
        OfficerRelationshipKind::ParentChild => "亲子",
        OfficerRelationshipKind::AdoptiveParentChild => "养亲子",
        OfficerRelationshipKind::Spouse => "夫妻",
        OfficerRelationshipKind::Sibling => "兄弟姊妹",
        OfficerRelationshipKind::SwornSibling => "结义",
        OfficerRelationshipKind::Enemy => "仇敌",
    }
}
