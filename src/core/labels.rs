use crate::game::*;

use super::i18n::Translator;

pub(super) fn development_focus_label(t: &Translator, focus: &DevelopmentFocus) -> String {
    match focus {
        DevelopmentFocus::Agriculture => t.text("development-focus-agriculture"),
        DevelopmentFocus::Commerce => t.text("development-focus-commerce"),
        DevelopmentFocus::Defense => t.text("development-focus-defense"),
        DevelopmentFocus::Order => t.text("development-focus-order"),
    }
}

pub(super) fn diplomacy_label(t: &Translator, proposal: &DiplomacyProposal) -> String {
    match proposal {
        DiplomacyProposal::ImproveRelations => t.text("diplomacy-improve-relations"),
        DiplomacyProposal::Truce => t.text("diplomacy-truce"),
        DiplomacyProposal::DeclareWar => t.text("diplomacy-declare-war"),
    }
}

pub(super) fn facility_kind_label(t: &Translator, kind: FacilityKind) -> String {
    match kind {
        FacilityKind::Farmland => t.text("facility-farmland"),
        FacilityKind::Irrigation => t.text("facility-irrigation"),
        FacilityKind::Market => t.text("facility-market"),
        FacilityKind::TradeDepot => t.text("facility-trade-depot"),
        FacilityKind::Workshop => t.text("facility-workshop"),
        FacilityKind::Quarry => t.text("facility-quarry"),
        FacilityKind::Barracks => t.text("facility-barracks"),
        FacilityKind::DrillGround => t.text("facility-drill-ground"),
        FacilityKind::Walls => t.text("facility-walls"),
        FacilityKind::Administration => t.text("facility-administration"),
        FacilityKind::Granary => t.text("facility-granary"),
        FacilityKind::RelayStation => t.text("facility-relay-station"),
        FacilityKind::Medical => t.text("facility-medical"),
    }
}

pub(super) fn troop_kind_label(t: &Translator, kind: TroopKind) -> String {
    match kind {
        TroopKind::Infantry => t.text("troop-infantry"),
        TroopKind::Cavalry => t.text("troop-cavalry"),
        TroopKind::Archers => t.text("troop-archers"),
    }
}

pub(super) fn technology_branch_label(t: &Translator, branch: TechnologyBranch) -> String {
    match branch {
        TechnologyBranch::Military => t.text("technology-branch-military"),
        TechnologyBranch::Domestic => t.text("technology-branch-domestic"),
    }
}

pub(super) fn confidence_label(t: &Translator, confidence: &SourceConfidence) -> String {
    match confidence {
        SourceConfidence::High => t.text("confidence-high"),
        SourceConfidence::Medium => t.text("confidence-medium"),
        SourceConfidence::Low => t.text("confidence-low"),
    }
}

pub(super) fn officer_gender_label(t: &Translator, gender: &OfficerGender) -> String {
    match gender {
        OfficerGender::Male => t.text("gender-male"),
        OfficerGender::Female => t.text("gender-female"),
    }
}

pub(super) fn officer_relationship_label(t: &Translator, kind: &OfficerRelationshipKind) -> String {
    match kind {
        OfficerRelationshipKind::RulerSubject => t.text("relationship-ruler-subject"),
        OfficerRelationshipKind::ParentChild => t.text("relationship-parent-child"),
        OfficerRelationshipKind::AdoptiveParentChild => {
            t.text("relationship-adoptive-parent-child")
        }
        OfficerRelationshipKind::Spouse => t.text("relationship-spouse"),
        OfficerRelationshipKind::Sibling => t.text("relationship-sibling"),
        OfficerRelationshipKind::SwornSibling => t.text("relationship-sworn-sibling"),
        OfficerRelationshipKind::Enemy => t.text("relationship-enemy"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::i18n::{Translator, UiLanguage};

    #[test]
    fn enum_labels_translate_in_chinese_and_english() {
        let zh = Translator::new(UiLanguage::SimplifiedChinese);
        let en = Translator::new(UiLanguage::English);

        assert_eq!(
            development_focus_label(&zh, &DevelopmentFocus::Agriculture),
            "农业"
        );
        assert_eq!(
            development_focus_label(&en, &DevelopmentFocus::Agriculture),
            "Agriculture"
        );
        assert_eq!(troop_kind_label(&zh, TroopKind::Cavalry), "骑兵");
        assert_eq!(troop_kind_label(&en, TroopKind::Cavalry), "Cavalry");
        assert_eq!(diplomacy_label(&zh, &DiplomacyProposal::DeclareWar), "宣战");
        assert_eq!(
            diplomacy_label(&en, &DiplomacyProposal::DeclareWar),
            "Declare War"
        );
        assert_eq!(confidence_label(&zh, &SourceConfidence::High), "高");
        assert_eq!(confidence_label(&en, &SourceConfidence::High), "High");
    }
}
