/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package tunnel

import "squirrel/arg"

func CLI() {
	opt := arg.Get(0)
	arg.Remove(0)

	switch opt {
	case "access":
		name := arg.Get()
		if name != "" {
			runAccess(name)
		}

	case "watchdog":
		name := arg.Get()
		if name != "" {
			runWatchdog(name)
		}

	case "worker":
		name := arg.Get()
		if name != "" {
			runWorker(name)
		}

	default:
		interactiveMenu(opt == "--ssh")
	}
}
