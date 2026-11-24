"""Opt-in Linux accessibility test. Operates only on the running SystemPulse UI.

Start SystemPulse with --widget, then run with --confirm-ui-test. No screenshots,
metrics, other application contents or agent configurations are read or printed.
Leaves the compact capsule visible and the dashboard hidden.
"""
import argparse
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--confirm-ui-test', action='store_true')
    args = parser.parse_args()
    if not args.confirm_ui_test:
        parser.error('Explicit approval required: --confirm-ui-test')
    import gi
    gi.require_version('Atspi', '2.0')
    from gi.repository import Atspi
    desktop = Atspi.get_desktop(0)
    apps = [desktop.get_child_at_index(i) for i in range(desktop.get_child_count())
            if desktop.get_child_at_index(i).get_name() == 'systempulse-desktop']
    assert len(apps) == 1, 'Exactly one SystemPulse instance required'
    app = apps[0]

    def walk(root):
        queue = [(root, 0)]
        visited = 0
        while queue and visited < 1000:
            node, depth = queue.pop(0)
            visited += 1
            yield node
            if depth < 18:
                queue.extend((node.get_child_at_index(i), depth + 1)
                             for i in range(node.get_child_count()))

    visible = lambda node: node.get_state_set().contains(Atspi.StateType.SHOWING)
    frames = [n for n in walk(app) if n.get_role_name() == 'frame']
    dashboard = next((n for n in frames if n.get_name() == 'SystemPulse'), None)
    capsule = next(n for n in frames if n.get_name() == 'SystemPulse Capsule')
    assert visible(capsule) and (dashboard is None or not visible(dashboard)), 'Start with --widget'

    def press(root, names):
        button = next(n for n in walk(root) if n.get_role_name() == 'button' and n.get_name() in names)
        assert button.get_action_iface().do_action(0)

    def wait(predicate):
        for _ in range(30):
            if predicate():
                return
            time.sleep(.1)
        raise AssertionError('Native UI state timeout')

    expanded = ('Collapse details', 'Réduire les détails')
    collapsed = ('Show details', 'Afficher les détails')
    toggle_named = lambda names: any(n.get_role_name() == 'button' and n.get_name() in names for n in walk(capsule))
    initial = capsule.get_component_iface().get_extents(Atspi.CoordType.SCREEN)
    press(capsule, collapsed)
    wait(lambda: toggle_named(expanded))
    large = capsule.get_component_iface().get_extents(Atspi.CoordType.SCREEN)
    assert large.height > initial.height
    press(capsule, ('Dashboard', 'Tableau de bord'))
    time.sleep(.5)
    dashboard = next(n for n in walk(app) if n.get_role_name() == 'frame' and n.get_name() == 'SystemPulse')
    wait(lambda: visible(dashboard))
    press(capsule, ('Hide capsule', 'Masquer la capsule'))
    wait(lambda: not visible(capsule))
    assert visible(dashboard)
    press(dashboard, ('Monitor capsule', 'Capsule de surveillance'))
    wait(lambda: visible(capsule))
    press(capsule, expanded)
    wait(lambda: toggle_named(collapsed))
    time.sleep(.5)
    final = capsule.get_component_iface().get_extents(Atspi.CoordType.SCREEN)
    press(dashboard, ('Close', 'Fermer'))
    wait(lambda: not visible(dashboard))
    assert visible(capsule)
    print('PASS native --widget, expand/collapse, hide/show, restore dashboard and close lifecycle')
    print({'compact_size': (initial.width, initial.height), 'expanded_size': (large.width, large.height),
           'final_size': (final.width, final.height)})


if __name__ == '__main__':
    main()
