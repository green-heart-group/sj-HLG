use crate::{cards::*, character::*, content::*, game::*};
#[test]
fn map_keeps_source_and_corrections() {
    assert_eq!(
        NODES.iter().filter(|n| n.exists(World::Surface)).count(),
        68
    );
    assert_eq!(
        (
            NODES[node("gate")].x,
            NODES[node("gate")].y,
            NODES[node("gate")].z
        ),
        (5, 0, 1)
    );
    assert_eq!(NODES[node("bio_office")].x, 2);
    for (key, z) in [
        ("chem_lab", 1),
        ("physics_lab", 2),
        ("electric_lab", 3),
        ("bio_lab", 4),
    ] {
        assert_eq!(NODES[node(key)].z, z);
    }
    assert!(NODES
        .iter()
        .filter(|n| n.exists(World::Surface))
        .all(|n| n.source.starts_with("TXT:")));
}
#[test]
fn original_achievement_catalog_is_complete() {
    assert_eq!(ACHIEVEMENTS.len(), 101);
    assert_eq!(ACHIEVEMENTS.iter().filter(|a| !a.root).count(), 99);
    let a = ACHIEVEMENTS.iter().find(|a| a.name == "手工艺人").unwrap();
    assert_eq!(a.text, "由于学业略有繁忙，\n你许久没有做过手工了。");
    assert!(ACHIEVEMENTS
        .iter()
        .any(|a| a.name == "化身为皂" && a.revision_marker == "&"));
}
#[test]
fn character_budget_and_growth_are_enforced() {
    assert!(Character::new([5, 5, 5, 5]).is_err());
    let mut c = Character::new([5, 3, 2, 2]).unwrap();
    assert!(!c.train(Attribute::Body));
    c.gain(8);
    assert_eq!(c.growth, 1);
    assert!(c.train(Attribute::Body));
    assert_eq!(c.values[0], 6);
    assert_eq!(c.growth, 0);
}
#[test]
fn illegal_movement_cannot_charge_or_teleport() {
    let mut g = Game::new(42, false);
    let before = g.clone();
    assert!(!g.apply(Action::Travel {
        to: node("principal"),
        mode: TravelMode::Walk
    }));
    assert_eq!(g.current, before.current);
    assert_eq!(g.hp, before.hp);
    assert_eq!(g.turn, 0);
}
#[test]
fn empty_hands_are_required_for_specimen() {
    let mut g = Game::new(42, false);
    g.current = node("specimen");
    g.world = World::Echo;
    g.give(Item::Lamp, 1);
    g.equip = Some(Item::Lamp);
    assert!(!g.apply(Action::Quest("diploid".into())));
    assert!(!g.has(Item::Diploid));
    assert!(g.apply(Action::Equip(None)));
    assert!(g.apply(Action::Quest("diploid".into())));
    assert!(g.has(Item::Diploid));
}
#[test]
fn crafting_consumes_only_declared_materials() {
    let mut g = Game::new(11, false);
    g.give(Item::Ink, 1);
    g.give(Item::Solution, 1);
    assert!(!g.apply(Action::Craft(Item::Lamp)));
    assert!(g.has(Item::Ink));
    g.give(Item::Matches, 1);
    assert!(g.apply(Action::Craft(Item::Lamp)));
    assert!(g.has(Item::Lamp));
    assert!(!g.has(Item::Ink));
    assert!(!g.has(Item::Matches));
    assert!(!g.has(Item::Solution));
}
#[test]
fn all_six_ending_priorities_match_source() {
    let mut g = Game::new(1, false);
    assert_eq!(g.ending(), Ending::Loop);
    g.mark("used_record");
    assert_eq!(g.ending(), Ending::Publish);
    g.flags.remove("used_record");
    g.mark("used_void");
    assert_eq!(g.ending(), Ending::Escape);
    g.counts.insert("thunder_favors".into(), 4);
    assert_eq!(g.ending(), Ending::Withered);
    g.mark("made_dark");
    assert_eq!(g.ending(), Ending::Annihilation);
    g.give(Item::Tear, 6);
    assert_eq!(g.ending(), Ending::Annihilation);
    g.give(Item::Tear, 1);
    assert_eq!(g.ending(), Ending::Mercy);
}
#[test]
fn entry_conditions_are_checked_on_teleport_cards() {
    let mut g = Game::new(12, false);
    g.current = node("chem_lab");
    g.give(Item::Thunder, 1);
    g.equip = Some(Item::Thunder);
    assert!(g.move_modes(node("bio_lab")).contains(&TravelMode::Leap));
    assert!(g.apply(Action::Travel {
        to: node("bio_lab"),
        mode: TravelMode::Leap
    }));
    assert_eq!(g.stamina, g.max_stamina() - 1);
    assert!(!NODES.iter().any(|n| n.exists(World::Surface) && n.roof()));
    g.world = World::Echo;
    g.current = *SOURCE_NODE_COUNT + 63;
    g.give(Item::Wheel, 1);
    g.equip = Some(Item::Wheel);
    let target = *SOURCE_NODE_COUNT + 64 + 63;
    assert!(!g.apply(Action::Travel {
        to: target,
        mode: TravelMode::Fly
    }));
    g.give(Item::Fuel, 1);
    assert!(g.apply(Action::Travel {
        to: target,
        mode: TravelMode::Fly
    }));
    assert_eq!(g.current, target);
    assert_eq!(g.count(Item::Fuel), 0);
}
#[test]
fn support_hand_has_real_consumption_and_persistent_state() {
    let mut g = Game::new(17, false);
    let before = g.clone();
    let t = g.hand[0];
    assert!(g.play(Action::Explore(Approach::Search), Some(t)));
    assert_eq!(g.hand.len(), 3);
    assert_eq!(g.discard_pile, vec![t]);
    assert_eq!(g.draw_pile.len(), 6);
    let restored: Game = serde_json::from_slice(&serde_json::to_vec(&g).unwrap()).unwrap();
    assert_eq!(restored, g);
    assert_ne!(g.rng, before.rng);
    g.validate().unwrap();
}
#[test]
fn a_rejected_action_does_not_consume_support() {
    let mut g = Game::new(17, false);
    let before = g.clone();
    assert!(!g.play(Action::Craft(Item::Lamp), Some(g.hand[0])));
    assert_eq!(g.hand, before.hand);
    assert_eq!(g.draw_pile, before.draw_pile);
    assert_eq!(g.turn, 0);
}
#[test]
fn result_receipt_is_computed_from_actual_differences() {
    let mut g = Game::new(9, false);
    let before = g.clone();
    assert!(g.apply(Action::Use(Item::Water)));
    let r = crate::feedback::Receipt::capture(&before, &g);
    assert!(!r.changes.iter().any(|s| s.starts_with("生命 ")));
    assert!(r.changes.iter().any(|s| s.contains("矿泉水 -1")));
    assert!(r.changes.iter().any(|s| s.contains("空瓶 +1")));
}
#[test]
fn death_preserves_plot_tokens_and_saved_ordinary_inventory() {
    let mut g = Game::new(9, false);
    g.give(Item::Lamp, 1);
    g.mail = Some(g.inv.clone());
    g.give(Item::Wenjin, 1);
    g.mark("met_wxh");
    g.mark("saponified");
    g.counts.insert("cold_until".into(), 30);
    g.give(Item::Coal, 5);
    g.rebirth("测试死亡", false);
    assert_eq!(g.current, node("clock"));
    assert!(g.has(Item::Lamp));
    assert!(g.has(Item::Wenjin));
    assert!(!g.has(Item::Coal));
    assert!(g.flag("met_wxh"));
    assert!(!g.flag("saponified"));
    assert_eq!(g.counter("cold_until"), 0);
    g.validate().unwrap();
}
fn play(g: &mut Game, a: Action) {
    let desired = match a {
        Action::Travel { .. } | Action::Use(Item::Hook) => Some(Tactic::Quiet),
        Action::Attack(_) => Some(Tactic::Opening),
        Action::Shift | Action::Quest(_) => Some(Tactic::Composure),
        _ => None,
    };
    let tactic = desired.filter(|t| g.hand.contains(t));
    assert!(
        g.play(a.clone(), tactic),
        "seed {} at {} {:?} rejected: {:?}",
        g.seed,
        NODES[g.current].id,
        a,
        g.last.notes
    );
    g.validate().unwrap();
    if g.character.growth > 0 {
        let attr = ATTRIBUTES
            .iter()
            .copied()
            .find(|&a| g.character.stat(a) < 6)
            .unwrap_or(Attribute::Will);
        if g.character.stat(attr) < 8 {
            assert!(g.apply(Action::Train(attr)));
        }
    }
}
fn walk(g: &mut Game, target: usize) {
    let original_world = g.world;
    for _ in 0..15000 {
        if g.world != original_world {
            return;
        }
        if g.current == target {
            return;
        }
        if !g.in_combat() {
            if g.stamina < 5
                || NODES[g.current].safe && (g.hp < g.character.max_hp() - 3 || g.sanity < 92)
            {
                play(g, Action::Rest);
                continue;
            }
            if g.world == World::Echo && g.sanity < 75 {
                play(g, Action::Explore(Approach::Study));
                continue;
            }
        } else if g.stamina < 3 && g.has(Item::Food) {
            play(g, Action::Use(Item::Food));
            continue;
        }
        let next = g.shortest_step(g.current, target).unwrap_or_else(|| {
            panic!(
                "seed {} no path {} -> {}",
                g.seed, NODES[g.current].id, NODES[target].id
            )
        });
        play(
            g,
            Action::Travel {
                to: next,
                mode: TravelMode::Walk,
            },
        );
    }
    panic!(
        "navigation exceeded cap: seed {} {} -> {}",
        g.seed, NODES[g.current].id, NODES[target].id
    )
}
fn go(g: &mut Game, key: &str, world: World) {
    for _ in 0..20 {
        if g.world != world {
            let mirror = g.nearest_mirror();
            walk(g, mirror);
            while g.world == World::Surface && g.sanity < 30 {
                play(g, Action::Rest);
            }
            play(g, Action::Shift);
        }
        walk(g, node(key));
        if g.current == node(key) && g.world == world {
            return;
        }
    }
    panic!("repeated deaths blocked route {key}, seed {}", g.seed)
}
fn snapshot(name: &str, g: &Game) {
    if let Some(dir) = std::env::var_os("HLG_QA_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{name}.json")),
            serde_json::to_vec_pretty(g).unwrap(),
        )
        .unwrap();
    }
}
fn run_route(seed: u64, easy: bool) -> Game {
    let mut g = Game::with_character(seed, easy, Character::new([4, 3, 3, 2]).unwrap());
    go(&mut g, "reception", World::Surface);
    play(&mut g, Action::Quest("guide".into()));
    go(&mut g, "lockers", World::Surface);
    play(&mut g, Action::Quest("lockers".into()));
    play(&mut g, Action::Craft(Item::Lamp));
    play(&mut g, Action::Equip(Some(Item::Lamp)));
    go(&mut g, "clock", World::Surface);
    play(&mut g, Action::SaveMail);
    go(&mut g, "company", World::Surface);
    play(&mut g, Action::Quest("company".into()));
    go(&mut g, "chem_office", World::Surface);
    if seed == 101 {
        snapshot("v2-before-card", &g);
    }
    play(&mut g, Action::Quest("card".into()));
    if seed == 101 {
        snapshot("v2-combat", &g);
    }
    play(&mut g, Action::Attack(AttackMode::Lamp));
    go(&mut g, "clock", World::Surface);
    play(&mut g, Action::SaveMail);
    go(&mut g, "classroom", World::Surface);
    play(&mut g, Action::Quest("wenjin".into()));
    play(&mut g, Action::Equip(Some(Item::Wenjin)));
    go(&mut g, "clock", World::Surface);
    play(&mut g, Action::SaveMail);
    go(&mut g, "fountain", World::Surface);
    play(&mut g, Action::Quest("mirror".into()));
    go(&mut g, "garden", World::Surface);
    play(&mut g, Action::Quest("hook".into()));
    play(&mut g, Action::Quest("leaf".into()));
    go(&mut g, "dungeon", World::Surface);
    for _ in 0..40 {
        if g.has(Item::Stroke) {
            break;
        }
        play(&mut g, Action::Use(Item::Hook));
    }
    assert!(g.has(Item::Stroke));
    go(&mut g, "physics_lab", World::Surface);
    play(&mut g, Action::Quest("thunder".into()));
    go(&mut g, "clock", World::Surface);
    play(&mut g, Action::SaveMail);
    go(&mut g, "director", World::Echo);
    play(&mut g, Action::Quest("memo".into()));
    play(&mut g, Action::Equip(Some(Item::Sword)));
    for _ in 0..60 {
        if g.has(Item::Fat) {
            break;
        }
        if g.stamina < 2 && g.has(Item::Food) {
            play(&mut g, Action::Use(Item::Food));
        } else {
            play(&mut g, Action::Attack(AttackMode::Strike));
        }
    }
    assert!(g.has(Item::Fat));
    go(&mut g, "chem_inner", World::Surface);
    play(&mut g, Action::Quest("mold".into()));
    play(&mut g, Action::Craft(Item::RoughSoap));
    let wxh = g
        .enemies
        .iter()
        .find(|e| e.kind == EnemyKind::Wxh && e.world == World::Surface)
        .unwrap()
        .node;
    walk(&mut g, wxh);
    while !g.in_combat() {
        play(&mut g, Action::Rest);
    }
    play(&mut g, Action::Use(Item::RoughSoap));
    assert!(g.flag("soap_story"));
    go(&mut g, "saponify", World::Echo);
    for _ in 0..150 {
        if g.has(Item::Soap) {
            break;
        }
        play(&mut g, Action::Quest("soap_trial".into()));
    }
    assert!(g.has(Item::Soap));
    go(&mut g, "specimen", World::Echo);
    if g.equip.is_some() {
        play(&mut g, Action::Equip(None));
    }
    play(&mut g, Action::Quest("diploid".into()));
    play(&mut g, Action::Equip(Some(Item::Diploid)));
    go(&mut g, "training", World::Echo);
    play(&mut g, Action::Quest("wheel".into()));
    if seed == 101 {
        snapshot("v2-echo", &g);
        let mut rogue = g.clone();
        let safe = rogue.nearest_matching(|n| NODES[n].safe).unwrap();
        walk(&mut rogue, safe);
        for _ in 0..12 {
            play(&mut rogue, Action::Explore(Approach::Search));
            if rogue.last.rolls.first().is_some_and(|r| r.success) {
                break;
            }
        }
        snapshot("v2-rogue", &rogue);
    }
    go(&mut g, "geo_office", World::Surface);
    play(&mut g, Action::Quest("ore".into()));
    play(&mut g, Action::Use(Item::Ore));
    for _ in 0..3 {
        play(&mut g, Action::Use(Item::Coal));
    }
    assert_eq!(g.relic_count(), 7, "seed {seed}");
    go(&mut g, "principal", World::Surface);
    for _ in 0..40 {
        if g.has(Item::Domain) {
            break;
        }
        go(&mut g, "principal", World::Surface);
        while g.sanity < 40 {
            if g.stamina < 3 {
                play(&mut g, Action::Rest);
            }
            play(&mut g, Action::Explore(Approach::Study));
        }
        play(&mut g, Action::Quest("domain".into()));
    }
    assert_eq!(g.relic_count(), 8);
    if seed == 101 {
        snapshot("v2-complete", &g);
    }
    go(&mut g, "reception", World::Surface);
    play(&mut g, Action::Quest("deliver".into()));
    go(&mut g, "reception", World::Surface);
    play(&mut g, Action::Quest("deliver".into()));
    go(&mut g, "gate", World::Surface);
    play(&mut g, Action::Quest("leave".into()));
    assert_eq!(g.status, Status::Ended(Ending::Loop));
    if seed == 101 {
        snapshot("v2-ending", &g);
    }
    g
}
#[test]
fn complete_surface_echo_card_route_uses_only_public_actions() {
    let mut receipts = vec![];
    for (seed, easy) in [
        (101, false),
        (117, false),
        (503, false),
        (907, false),
        (1103, true),
        (2209, true),
    ] {
        let g = run_route(seed, easy);
        receipts.push(serde_json::json!({"seed":seed,"easy":easy,"turns":g.turn,"deaths":g.counter("deaths"),"relics":g.relic_count(),"level":g.character.level,"ending":"循环","public_actions_only":true,"echo_rooms":NODES.len(),"echo_revisions":g.echo.revision,"build_fingerprint":env!("HLG_BUILD_FINGERPRINT")}));
    }
    if let Some(dir) = std::env::var_os("HLG_QA_DIR") {
        std::fs::write(
            std::path::PathBuf::from(dir).join("full-runs-v2.json"),
            serde_json::to_vec_pretty(&receipts).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn echo_seed_and_repeated_rearrangements_are_connected_and_reproducible() {
    let surface: Vec<_> = EDGES
        .iter()
        .filter(|e| e.worlds & 1 != 0)
        .map(|e| (e.a, e.b))
        .collect();
    for seed in [1, 17, 101, 1907, 99173] {
        let mut a = crate::echo::EchoLayout::new(seed);
        let mut b = crate::echo::EchoLayout::new(seed);
        assert_eq!(a, b);
        assert!(NODES.len() >= 1100);
        a.validate().unwrap();
        for step in 0..32 {
            let old = a.edges();
            let z = Some(step % 16);
            a.regenerate(z);
            b.regenerate(z);
            assert_eq!(a, b);
            assert_ne!(old, a.edges());
            a.validate().unwrap();
        }
    }
    assert_eq!(
        surface,
        EDGES
            .iter()
            .filter(|e| e.worlds & 1 != 0)
            .map(|e| (e.a, e.b))
            .collect::<Vec<_>>()
    );
}
#[test]
fn changing_world_preserves_inventory_plot_and_current_room() {
    let mut g = Game::new(101, false);
    g.world = World::Echo;
    g.mark("mirror");
    g.give(Item::Wenjin, 1);
    g.mark("saved_soap_student");
    g.survey_echo();
    let old = g.clone();
    assert!(g.apply(Action::Reshape));
    assert_eq!(g.current, old.current);
    assert_eq!(g.count(Item::Wenjin), 1);
    assert!(g.flag("saved_soap_student"));
    assert!(g.echo.revision > old.echo.revision);
    assert!(g.known[1].len() < old.known[1].len());
    assert!(g.last.changes.iter().any(|x| x.contains("地图重组")));
    g.validate().unwrap();
    let save = serde_json::to_vec(&g).unwrap();
    assert!(save.len() < 3_000_000);
    let restored: Game = serde_json::from_slice(&save).unwrap();
    assert_eq!(g, restored);
}
#[test]
fn crossing_zone_changes_departed_routes_without_stranding_player() {
    let mut g = Game::new(101, false);
    g.world = World::Echo;
    g.current = g.echo.cells[0][4 * 8 + 7];
    g.reveal();
    let next = g
        .neighbors(g.current)
        .into_iter()
        .find(|&n| g.echo.zone(n) == 1)
        .unwrap();
    let before = g.echo.clone();
    assert!(g.apply(Action::Travel {
        to: next,
        mode: TravelMode::Walk
    }));
    assert_eq!(g.current, next);
    assert_eq!(g.echo.epochs[0], before.epochs[0] + 1);
    assert_eq!(g.echo.epochs[1], before.epochs[1]);
    g.validate().unwrap();
    assert!(g.shortest_step(g.current, node("specimen")).is_some());
}
#[test]
fn periodic_echo_rearrangement_and_mirror_return_use_real_actions() {
    let mut g = Game::new(101, false);
    g.world = World::Echo;
    g.current = *SOURCE_NODE_COUNT + 1;
    g.mark("mirror");
    g.entry_surface = node("fountain");
    g.counts.insert("echo_actions".into(), 23);
    let rev = g.echo.revision;
    assert!(g.apply(Action::Rest));
    assert_eq!(g.echo.revision, rev + 1);
    assert!(g.apply(Action::Shift));
    assert_eq!(g.world, World::Surface);
    assert_eq!(g.current, node("fountain"));
    g.validate().unwrap();
}
#[test]
fn support_and_preparation_change_the_actual_recorded_roll() {
    let mut g = Game::new(51, false);
    g.mark("prepared");
    g.hand = vec![Tactic::Observe, Tactic::Guard, Tactic::Quiet];
    g.draw_pile = vec![
        Tactic::Observe,
        Tactic::Guard,
        Tactic::Quiet,
        Tactic::Opening,
        Tactic::Opening,
        Tactic::Composure,
        Tactic::Composure,
    ];
    assert!(g.play(Action::Explore(Approach::Study), Some(Tactic::Observe)));
    assert_eq!(
        g.last.rolls[0].bonus,
        g.character.stat(Attribute::Insight) + 4
    );
    assert!(!g.flag("prepared"));
    g.validate().unwrap();
}
#[test]
fn tool_effects_and_consumption_apply_only_to_valid_targets() {
    let mut g = Game::new(51, false);
    g.current = node("chem_lab");
    g.previous = node("hall");
    g.give(Item::RoughSoap, 1);
    g.give(Item::Fusion, 1);
    assert!(!g.apply(Action::Use(Item::RoughSoap)));
    assert!(g.has(Item::RoughSoap));
    assert_eq!(g.turn, 0);
    g.spawn(EnemyKind::Wxh);
    g.enemies[0].stun = 0;
    assert!(g.apply(Action::Use(Item::RoughSoap)));
    assert!(!g.has(Item::RoughSoap));
    assert_ne!(g.enemies[0].node, g.current);
    assert!(g
        .last
        .changes
        .iter()
        .any(|s| s.contains("已移至") && s.contains("停攻")));
    assert!(g.flag("soap_story"));
    assert!(!g.apply(Action::Use(Item::Fusion)));
    assert!(g.has(Item::Fusion));
    g.enemies = vec![
        Enemy {
            kind: EnemyKind::Copy,
            node: g.current,
            world: g.world,
            hp: 12,
            stun: 0,
            chasing: false
        };
        2
    ];
    assert!(g.apply(Action::Use(Item::Fusion)));
    assert_eq!(g.enemies.iter().filter(|e| e.hp > 0).count(), 1);
    assert!(g
        .last
        .changes
        .iter()
        .any(|s| s.contains("生命 12 → 0") && s.contains("失去行动能力")));
    assert!(!g.has(Item::Fusion));
}
