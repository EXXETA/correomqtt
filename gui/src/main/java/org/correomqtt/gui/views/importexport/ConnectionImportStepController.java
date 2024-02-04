package org.correomqtt.gui.views.importexport;

import org.correomqtt.gui.views.base.BaseController;

public interface ConnectionImportStepController extends BaseController {
    void cleanUp();

    void initFromWizard();
}
