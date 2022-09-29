from django.urls import path
from lang.views import CompileView


urlpatterns = [path("compile", CompileView.as_view())]
