cbuffer cbPerObject : register(b0)
{
    float4x4 model_view_projection;
    // TODO: Z Index
};

struct VertexIn
{
    float2 position : POSITION;
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
    
    // TODO: Z Index
    result.position = mul(float4(vIn.position,0.0,1.0), model_view_projection);
    result.color = vIn.color;

    return result;
}

float4 PSMain(VertexOut input) : SV_TARGET
{
    return input.color;
}