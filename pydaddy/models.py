from django.db import models


# Create your models here.
class RemoteModule(models.Model):
    address64 = models.BinaryField(null=False)
    node_id = models.CharField(max_length=64, null=False)
    operating_mode = models.BinaryField(null=False)
    network_id = models.BinaryField(null=False)
    parent_device = models.BinaryField(null=False)

    class Meta:
        db_table = "remote_modules"