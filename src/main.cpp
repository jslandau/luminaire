#include <QApplication>
#include "MainWindow.h"

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    app.setApplicationName("Luminaire");
    app.setOrganizationName("Luminaire");

    bool minimized = app.arguments().contains("--minimized");
#ifdef Q_OS_MACOS
    minimized = true;
#endif

    MainWindow window;
    if (!minimized) {
        window.show();
    }

    return app.exec();
}
