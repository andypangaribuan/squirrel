/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

pub(super) const SEARCH_FILE_MAX_LEVEL_ABOVE: usize = 4;
pub(super) const KEY_KYML_PV_NAME: &str = "KYML_PV_NAME";
pub(super) const KEY_KYML_PVC_NAME: &str = "KYML_PVC_NAME";
pub(super) const GITHUB_TEMPLATE_DIRECTORY: &str = "https://raw.githubusercontent.com/andypangaribuan/squirrel/refs/heads/main/template/";

pub(super) const COMMAND_ACTION_PODS: &[(&str, &str)] = &[
    ("ls", "show running pods"),
    ("watch", "stream every second of running pods"),
    ("rollout", "rolling update of application"),
    ("delete", "[+name] delete specific pod"),
    ("exec", "[+name] go to shell pod (default: first pod)"),
    ("scale", "[+int] scale deployment to [int] size"),
    ("logs", "[+since] stream pods log, (default) since: 60m"),
    ("events", "stream pods events"),
];
