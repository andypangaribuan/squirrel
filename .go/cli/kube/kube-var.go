/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package kube

const searchFileMaxLevelAbove = 4
const singleSpace = " "
const doubleSpace = "  "
const tripleSpace = "   "

const keyKymlPvName = "KYML_PV_NAME"
const keyKymlPvcName = "KYML_PVC_NAME"

var commandActionPods = [][]string{
	{"ls", "show running pods"},
	{"watch", "stream every second of running pods"},
	{"rollout", "rolling update of application"},
	{"delete", "[+name] delete specific pod"},
	{"exec", "[+name] go to shell pod (default: first pod)"},
	{"scale", "[+int] scale deployment to [int] size"},
	{"logs", "[+since] stream pods log, (default) since: 60m"},
	{"events", "stream pods events"},
}
