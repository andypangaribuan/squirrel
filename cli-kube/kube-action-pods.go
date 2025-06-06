/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package clikube

import "squirrel/app"

func kubeActionPods(namespace string, appName string) {
	args := app.Args

	switch {
	case args.IsLs:
		kubeActionPodsLs(namespace, appName)

	case args.IsWatch:
		kubeActionPodsWatch(namespace, appName)

	case args.IsRollout:
		kubeActionPodsRollout(namespace, appName)

	case args.IsDelete:
		kubeActionPodsDelete(namespace, appName)

	case args.IsScale:
		kubeActionPodsScale(namespace, appName)

	case args.IsLogs:
		kubeActionPodsLogs(namespace, appName)

	case args.IsEvents:
		kubeActionPodsEvents(namespace, appName)
	}
}
