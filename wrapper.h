#ifndef WRAPPER_H_
#define WRAPPER_H_

// Needed to avoid the dreaded "error: expected ';' after top level declarator"
// error.
#define __kernel_ptr_semantics

/* include <CoreFoundation/CFDictionary.h> */
/* include <CoreFoundation/CFNumber.h> */
/* include <CoreFoundation/CFString.h> */
/* include <CoreFoundation/CFPlugInCOM.h> */
#include <IOKit/IOReturn.h>
#include <IOKit/IOMessage.h>
#include <IOKit/IOKitLib.h>
#include <IOKit/IOTypes.h>
#include <IOKit/usb/IOUSBLib.h>
#include <IOKit/pwr_mgt/IOPMLib.h>
#include <IOKit/IOCFPlugIn.h>

#endif // WRAPPER_H_
