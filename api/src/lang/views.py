from rest_framework import views
from rest_framework.response import Response
from lang.serializers import (
    CompileResultSerializer,
    CompileSourceSerializer,
)
import gdlk_pyo3


class CompileView(views.APIView):
    def post(self, request):
        serializer = CompileSourceSerializer(data=request.data)
        serializer.is_valid()
        data = serializer.validated_data
        return Response(
            CompileResultSerializer({"ast": gdlk_pyo3.sum_as_string(1, 2)}).data
        )
