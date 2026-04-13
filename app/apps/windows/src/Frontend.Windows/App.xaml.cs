using System.Windows;
using Frontend.Windows.Infrastructure.Api;
using Frontend.Windows.Infrastructure.Notifications;
using Frontend.Windows.Presentation.ViewModels;
using Frontend.Windows.Presentation.Views;
using System.Net.Http;

namespace Frontend.Windows;

public partial class App : Application
{
    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);
        try 
        {
            var view = new MainWindow();
            view.Show();
        }
        catch (Exception ex)
        {
            MessageBox.Show(ex.ToString()); // Это заставит ошибку показаться!
        }
    }
}