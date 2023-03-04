cbuffer cbPerObject : register(b0)
{
    float4x4 gWorld;
    float4x4 gViewProj;
};

struct VertexIn
{
    float4 position : POSITION;
    float4 color : COLOR;
};

struct VertexOut
{
    float4 position : SV_POSITION;
    float4 color : COLOR;
};

VertexOut VSMain(VertexIn vIn)
{
    VertexOut result;

    result.position = mul(vIn.position, gWorld);
    result.position = mul(result.position, gViewProj);
    result.color = vIn.color;

    return result;
}

float4 PSMain(VertexOut input) : SV_TARGET
{
    return input.color;
}