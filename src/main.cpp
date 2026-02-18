#include <QApplication>
#include "MainWindow.h"

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    app.setApplicationName("Luminaire");
    app.setOrganizationName("Luminaire");

    bool minimized = app.arguments().contains("--minimized");

    MainWindow window;
    if (!minimized) {
        window.show();
    }

    return app.exec();
}
