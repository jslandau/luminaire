#include <QApplication>
#include "MainWindow.h"

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    app.setApplicationName("Luminaire");
    app.setOrganizationName("Luminaire");

    MainWindow window;
    window.show();

    return app.exec();
}
